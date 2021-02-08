//
//  Gitlab.swift
//  
//
//  Created by Til Blechschmidt on 15.12.20.
//

import Foundation
import ArgumentParser

import SQLite

struct Gitlab: ParsableCommand {
    static var configuration = CommandConfiguration(abstract: "GitLab Webhook source")

    @OptionGroup var options: Ingest.Options

    func run() throws {
        let db = try Database(path: options.database)

        guard let reader = LineReader(path: options.input) else {
            throw Ingest.Error.UnableToReadInput
        }

        let eventSeparator = "--- ---\n"

        var stage: ParseStage = .unknown
        var timestamp = ""
        var event = ""
        var payload = ""
        var processedPipelines = Set<Int>()
        var eventCount = 0

        for (i, line) in reader.enumerated() {
            if line == eventSeparator {
                if stage != .unknown {
                    print("Unexpected event separator at line \(i)")
                }

                stage = .timestamp
            } else if stage == .timestamp {
                timestamp = line
                stage = .event
            } else if stage == .event {
                event = line
                stage = .payload
            } else if stage == .payload {
                payload = line
                stage = .unknown
                eventCount += 1

                do {
                    try autoreleasepool {
                        let event = try GitlabEvent(timestamp: timestamp, event: event, payload: payload)
                        try processEvent(event: event, db, &processedPipelines)
                    }
                } catch {
                    print("Failed to parse event at line \(i): \(error)")
                }
            }
        }

        // Speed up querying in the simulation :)
        try db.db.execute("""
            CREATE INDEX "JobIndex" ON "Pipeline" (
                "id"    ASC,
                "jobs",
                "status"
            )
        """)

        print("Processed \(eventCount) GitLab Events (raw input)")
    }

    func processEvent(event: GitlabEvent, _ db: Database, _ processedPipelines: inout Set<Int>) throws {
        // TODO For some reason Gitlab pushes multiple "completion" events for the same pipeline with different finishedAt and duration values (and apparently builds as well)
        //      1. Figure out what the hell is going on and why it does that
        //      2. Filter them out or merge the data
        // 258855
        // 258624
//        if case let .pipeline(event) = event, event.objectAttributes.id == 256926, event.objectAttributes.status.isCompleted {
//            print(event)
//        }

        // Pipelines that have run to completion, crashed or otherwise exited
        // Do not insert pipelines for which a completion event has already been processed. GitLab pushes multiple completion events with different values for some reason 🤷‍♂️
        if case let .pipeline(event) = event, event.objectAttributes.status.isCompleted, !processedPipelines.contains(event.objectAttributes.id) {
            processedPipelines.insert(event.objectAttributes.id)

            let pipelineID = Int64(event.objectAttributes.id)
            let createdTimestamp = event.objectAttributes.createdAt.flatMap { convert(gitlabDate: $0) }
            let finishedTimestamp = event.objectAttributes.finishedAt.flatMap { convert(gitlabDate: $0) }

            try db.db.run(db.pipelines.table.insert(or: .replace,
                db.pipelines.id <- pipelineID,
                db.pipelines.status <- event.objectAttributes.status.rawValue,
                db.pipelines.duration <- event.objectAttributes.duration.flatMap { Int64($0) },
                db.pipelines.createdAt <- createdTimestamp,
                db.pipelines.finishedAt <- finishedTimestamp,
                db.pipelines.ref <- event.objectAttributes.ref,
                db.pipelines.jobs <- event.testJobs.joined(separator: ";")
            ))

            if let createdTimestamp = createdTimestamp {
                try db.db.run(db.events.table.insert(
                    db.events.timestamp <- createdTimestamp,
                    db.events.kind <- SimulationEvent.Kind.pipelineCreated.rawValue,
                    db.events.key <- pipelineID
                ))
            }

            if let finishedTimestamp = finishedTimestamp {
                try db.db.run(db.events.table.insert(
                    db.events.timestamp <- finishedTimestamp,
                    db.events.kind <- SimulationEvent.Kind.pipelineFinished.rawValue,
                    db.events.key <- pipelineID
                ))
            }
        }

        // Merge requests
        if case let .merge(event) = event {
            let timestamp = convert(gitlabDate: event.objectAttributes.updatedAt)

            let eventID = try db.db.run(db.mergeRequestEvents.table.insert(
                db.mergeRequestEvents.mergeRequestID <- Int64(event.objectAttributes.id),
                db.mergeRequestEvents.status <- event.objectAttributes.state.rawValue,
                db.mergeRequestEvents.sourceBranch <- event.objectAttributes.sourceBranch,
                db.mergeRequestEvents.targetBranch <- event.objectAttributes.targetBranch,
                db.mergeRequestEvents.action <- event.objectAttributes.action.rawValue,
                db.mergeRequestEvents.timestamp <- timestamp
            ))

            try db.db.run(db.events.table.insert(
                db.events.timestamp <- timestamp,
                db.events.kind <- SimulationEvent.Kind.mergeRequestEvent.rawValue,
                db.events.key <- eventID
            ))
        }
    }
}

fileprivate func convert(gitlabDate: String) -> Int64 {
//    let targetFormatter = ISO8601DateFormatter()
    let sourceFormatter = DateFormatter()
    sourceFormatter.dateFormat = "yyyy-MM-dd HH:mm:ss Z"
    sourceFormatter.locale = Locale(identifier: "en_US_POSIX")

    let date = sourceFormatter.date(from: gitlabDate)!
    return Int64(date.timeIntervalSince(cutoffDate)) // targetFormatter.string(from: date)
}
