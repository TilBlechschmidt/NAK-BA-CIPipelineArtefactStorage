//
//  File.swift
//  
//
//  Created by Til Blechschmidt on 15.12.20.
//

import Foundation
import ArgumentParser

import SQLite

struct Nginx: ParsableCommand {
    static var configuration = CommandConfiguration(abstract: "Nginx web server source")

    @OptionGroup var options: Ingest.Options

    func run() throws {
        let db = try Database(path: options.database)

        guard let reader = LineReader(path: options.input) else {
            throw Ingest.Error.UnableToReadInput
        }

        let cutoffDate = Date(timeIntervalSince1970: 1608049887) // 2020-12-15 16:31:26 +0000
        var entryCount = 0
        var failedCount = 0
        var excludedCount = 0

        for (i, line) in reader.enumerated() {
            let progress = Double(reader.position) / Double(reader.size) * 100
            let formattedProgress = String(format: "%03.2f", arguments: [progress])
            print("\r\(formattedProgress) %", terminator: "")

            do {
                entryCount += 1
                try autoreleasepool {
                    let entry = try LogEntry(from: line)
                    if entry.timestamp >= cutoffDate {
                        let timestamp = Int64(entry.timestamp.timeIntervalSince(cutoffDate))

                        let accessID = try db.db.run(db.accessLog.table.insert(
                            db.accessLog.timestamp <- timestamp,

                            db.accessLog.method <- String(entry.method),
                            db.accessLog.path <- String(entry.path),

                            db.accessLog.status <- entry.status,
                            db.accessLog.bytes <- entry.bytes,

                            db.accessLog.referee <- String(entry.referee),
                            db.accessLog.userAgent <- String(entry.userAgent),

                            db.accessLog.isAutomatic <- entry.isAutomaticRequest,
                            db.accessLog.isIrrelevant <- entry.isIrrelevantRequest,

                            db.accessLog.repository <- entry.repository,
                            db.accessLog.pipeline <- entry.pipeline,
                            db.accessLog.job <- entry.job,
                            db.accessLog.file <- entry.file
                        ))

                        if !entry.isAutomaticRequest && !entry.isIrrelevantRequest && entry.pipeline != nil {
                            try db.db.run(db.events.table.insert(
                                db.events.timestamp <- timestamp,
                                db.events.kind <- SimulationEvent.Kind.access.rawValue,
                                db.events.key <- accessID
                            ))
                        }

                        if let pipelineID = entry.pipeline {
                            try db.db.run(db.pipelines.table.insert(or: .ignore, db.pipelines.id <- pipelineID))
                        }
                    } else {
                        excludedCount += 1
                    }
                }
            } catch {
                failedCount += 1
                print("Failed to parse line \(i) from input")
            }
        }

        print("Parsed \(entryCount) access events (excluded \(excludedCount), failed \(failedCount)")
    }
}
