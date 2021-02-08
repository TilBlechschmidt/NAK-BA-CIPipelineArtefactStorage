//
//  GitlabEvent.swift
//  
//
//  Created by Til Blechschmidt on 01.01.21.
//

import Foundation

enum ParseStage {
    case unknown
    case timestamp
    case event
    case payload
}

struct GitlabJobEvent {}

enum PipelineStatus: String, Decodable {
    case pending
    case running
    case success
    case failed
    case canceled
    case skipped

    // Build status
    case created
    case manual

    var isCompleted: Bool {
        return [.success, .failed, .canceled, .skipped].contains(self)
    }
}

struct GitlabPipelineObjectAttributes: Decodable {
    let id: Int
    let ref: String
    let tag: Bool
    let sha: String
    let source: String

    let status: PipelineStatus
    let stages: [String]

    let duration: Int?
    let createdAt: String?
    let finishedAt: String?

    let variables: [[String : String]]

    enum CodingKeys: String, CodingKey {
        case id
        case ref
        case tag
        case sha
        case source

        case status
        case stages

        case duration
        case createdAt = "created_at"
        case finishedAt = "finished_at"

        case variables
    }
}

struct GitlabPipelineBuild: Decodable {
    let id: Int
    let stage: String
    let name: String
    let status: PipelineStatus

    // Some names do contain a postfix like "1/3" indicating the current step, this function removes that
    var cleanedName: String {
        if let postfix = SimpleRegexMatcher.firstMatch(forPattern: " \\d+\\/\\d+$", in: name), let range = postfix.ranges[0] {
            var newName = name
            newName.replaceSubrange(range, with: [])
            return newName
        } else {
            return name
        }
    }
}

struct GitlabPipelineEvent: Decodable {
    let objectAttributes: GitlabPipelineObjectAttributes
//    let mergeRequest: String?
    let builds: [GitlabPipelineBuild]

    var testJobs: Set<String> {
        let rawJobs = builds
            .filter { $0.stage == "test" && $0.status.isCompleted }
            .map { $0.cleanedName }
            .filter { $0 != "test:shutdown-bazel-build-pod" }

        return Set(rawJobs)
    }

    enum CodingKeys: String, CodingKey {
        case objectAttributes = "object_attributes"
//        case mergeRequest = "merge_request"
        case builds
    }
}

enum MergeStatus: String, Decodable {
    case unchecked
    case checking
    case mergeConflict = "cannot_be_merged"
    case targetBranchChanged = "cannot_be_merged_recheck"
    case mergeable = "can_be_merged"
}

enum MergeRequestState: String, Decodable {
    case opened
    case merged
    case closed
}

enum MergeRequestAction: String, Decodable {
    case open
    case reopen
    case update
    case approved
    case unapproved
    case merge
    case close
}

struct GitlabMergeEventObjectAttributes: Decodable {
    let id: Int

    let mergeStatus: MergeStatus
    let state: MergeRequestState
    let action: MergeRequestAction
    let updatedAt: String

    let sourceBranch: String
    let targetBranch: String

    enum CodingKeys: String, CodingKey {
        case id = "id" // there seem to be two ids (id, iid)

        case mergeStatus = "merge_status"
        case state
        case action
        case updatedAt = "updated_at"

        case sourceBranch = "source_branch"
        case targetBranch = "target_branch"
    }
}

struct GitlabMergeEvent: Decodable {
    let objectAttributes: GitlabMergeEventObjectAttributes

    enum CodingKeys: String, CodingKey {
        case objectAttributes = "object_attributes"
    }
}

struct GitlabPushEvent {}
struct GitlabTagPushEvent {}

enum GitlabEventParseError: Error {
    case invalidHeader
    case invalidTimestamp
    case unknownEventType
    case beforeCutoffDate
}

enum GitlabEvent {
    case job(GitlabJobEvent)
    case pipeline(GitlabPipelineEvent)
    case merge(GitlabMergeEvent)
    case push(GitlabPushEvent)
    case tagPush(GitlabTagPushEvent)

    init(timestamp: String, event: String, payload: String) throws {
        guard timestamp.hasPrefix("Timestamp:") && event.hasPrefix("X-Gitlab-Event:") else {
            throw GitlabEventParseError.invalidHeader
        }

        let timestamp = timestamp.dropFirst("Timestamp:".count).trimmingCharacters(in: .whitespacesAndNewlines)
        let event = event.dropFirst("X-Gitlab-Event:".count).trimmingCharacters(in: .whitespacesAndNewlines)

        guard let timestampInterval = TimeInterval(timestamp) else {
            throw GitlabEventParseError.invalidTimestamp
        }

        let date = Date(timeIntervalSince1970: timestampInterval)

        if date < cutoffDate {
            throw GitlabEventParseError.beforeCutoffDate
        }

        switch event {
        case "Job Hook":
            self = .job(GitlabJobEvent())
        case "Push Hook":
            self = .push(GitlabPushEvent())
        case "Pipeline Hook":
            let event = try JSONDecoder().decode(GitlabPipelineEvent.self, from: payload.data(using: .utf8)!)
            self = .pipeline(event)
        case "Merge Request Hook":
            let event = try JSONDecoder().decode(GitlabMergeEvent.self, from: payload.data(using: .utf8)!)
            self = .merge(event)
        case "Tag Push Hook":
            self = .tagPush(GitlabTagPushEvent())
        default:
            throw GitlabEventParseError.unknownEventType
        }
    }
}
