//
//  File.swift
//  
//
//  Created by Til Blechschmidt on 01.01.21.
//

import Foundation
import SQLite

struct SimulationEvent {
    enum Kind: Int {
        case pipelineCreated = 0
        case pipelineFinished = 1
        case mergeRequestEvent = 2
        case access = 3
    }

    let table = Table("SimulationEvent")

    let id = Expression<Int64>("id")

    let timestamp = Expression<Int64>("timestamp")
    let kind = Expression<Int>("kind")
    let key = Expression<Int64>("key")

    init(db: Connection) throws {
        try db.run(self.table.create(ifNotExists: true) { t in
            t.column(id, primaryKey: true)
            t.column(timestamp)
            t.column(kind)
            t.column(key)
        })
    }
}

struct JobSizeSample {
    let table = Table("JobSizeSample")

    let id = Expression<Int64>("id")

    let environment = Expression<String>("environment")
    let testSuite = Expression<String>("testSuite")

    let bytes = Expression<Int64>("bytes")

    init(db: Connection) throws {
        try db.run(self.table.create(ifNotExists: true) { t in
            t.column(id, primaryKey: true)
            t.column(environment)
            t.column(testSuite)
            t.column(bytes)
        })
    }
}

struct MergeRequestEvent {
    let table = Table("MergeRequestEvent")

    let eventID = Expression<Int64>("eventID")
    let mergeRequestID = Expression<Int64>("mergeRequestID")
    let status = Expression<String?>("status")

    let sourceBranch = Expression<String>("sourceBranch")
    let targetBranch = Expression<String>("targetBranch")

    let action = Expression<String>("action")
    let timestamp = Expression<Int64?>("timestamp")

    init(db: Connection) throws {
        try db.run(self.table.create(ifNotExists: true) { t in
            t.column(eventID, primaryKey: true)
            t.column(mergeRequestID)
            t.column(status)
            t.column(sourceBranch)
            t.column(targetBranch)
            t.column(action)
            t.column(timestamp)
        })
    }
}

struct Pipeline {
    let table = Table("Pipeline")

    let id = Expression<Int64>("id")
    let status = Expression<String?>("status")

    let duration = Expression<Int64?>("duration")
    let createdAt = Expression<Int64?>("createdAt")
    let finishedAt = Expression<Int64?>("finishedAt")

    let ref = Expression<String?>("ref")
    let jobs = Expression<String?>("jobs")

    init(db: Connection) throws {
        try db.run(self.table.create(ifNotExists: true) { t in
            t.column(id, primaryKey: true)
            t.column(status)
            t.column(duration)
            t.column(createdAt)
            t.column(finishedAt)
            t.column(ref)
            t.column(jobs)
        })
    }
}

struct AccessLog {
    let table = Table("AccessLog")

    let id = Expression<Int64>("id")
    let timestamp = Expression<Int64>("timestamp")

    let method = Expression<String>("method")
    let path = Expression<String>("path")

    let status = Expression<Int>("status")
    let bytes = Expression<Int>("bytes")

    let referee = Expression<String>("referee")
    let userAgent = Expression<String>("userAgent")

    let isAutomatic = Expression<Bool>("isAutomatic")
    let isIrrelevant = Expression<Bool>("isIrrelevant")

    let repository = Expression<String?>("repository")
    let pipeline = Expression<Int64?>("pipeline")
    let job = Expression<String?>("job")
    let file = Expression<String?>("file")

    init(db: Connection, pipeline: Pipeline) throws {
        try db.run(self.table.create(ifNotExists: true) { t in
            t.column(id, primaryKey: true)
            t.column(timestamp)

            t.column(method)
            t.column(path)

            t.column(status)
            t.column(bytes)

            t.column(referee)
            t.column(userAgent)

            t.column(isAutomatic)
            t.column(isIrrelevant)

            t.column(repository)
            t.column(self.pipeline, references: pipeline.table, pipeline.id)
            t.column(job)
            t.column(file)
        })
    }
}

struct Database {
    let db: Connection
    let pipelines: Pipeline
    let accessLog: AccessLog
    let mergeRequestEvents: MergeRequestEvent
    let jobSizeSamples: JobSizeSample
    let events: SimulationEvent

    init(path: String) throws {
        db = try Connection(path)
        pipelines = try Pipeline(db: db)
        accessLog = try AccessLog(db: db, pipeline: pipelines)
        mergeRequestEvents = try MergeRequestEvent(db: db)
        jobSizeSamples = try JobSizeSample(db: db)
        events = try SimulationEvent(db: db)

//        try db.execute("CREATE VIEW IF NOT EXISTS Branches AS SELECT ref FROM pipeline UNION SELECT sourceBranch FROM MergeRequestEvent");
    }
}
