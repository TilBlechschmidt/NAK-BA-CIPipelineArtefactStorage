//
//  File.swift
//  
//
//  Created by Til Blechschmidt on 15.12.20.
//

import Foundation

fileprivate var combinedLogPattern = "^(\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3})\\s-\\s([^\\s]+)\\s\\[([^\\]]+)] \"([^\" ]+) ([^\" ]+) ([^\" ]+)\" (\\d{3}) (\\d+) \"([^\"]+)\" \"([^\"]+)\"$"

fileprivate func clean(_ path: Substring) -> Substring {
    var newPath = path

    if let argsIndex = newPath.lastIndex(of: "?") {
        newPath = newPath[newPath.startIndex..<argsIndex]
    }

    if newPath.count > 1 && newPath.hasSuffix("/") {
        newPath = newPath[newPath.startIndex..<newPath.index(before: newPath.endIndex)]
    }

    return newPath
}

struct LogEntry {
    enum Error: Swift.Error {
        case lineNotMatching
    }

    static var dateFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "dd/MMM/yyyy:HH:mm:ss Z" // 27/Nov/2020:12:34:58 +0000
        return formatter
    }()

    let ip: Substring
    let remoteUser: Substring
    let rawTimestamp: Substring
    var timestamp: Date! { return LogEntry.dateFormatter.date(from: String(rawTimestamp)) }
    var isoTimestamp: String { return ISO8601DateFormatter().string(from: timestamp) }

    let method: Substring
    let path: Substring
    let proto: Substring

    let rawStatus: Substring
    var status: Int { return Int(rawStatus)! }
    let rawBytes: Substring
    var bytes: Int { return Int(rawBytes)! }

    let referee: Substring
    let userAgent: Substring

    var isAutomaticRequest: Bool {
        let isHtmlReportResource = SimpleRegexMatcher.firstMatch(forPattern: "(?:htmlReports|html_report)\\/.+", in: String(path)) != nil
        let isDirectoryListingMetadata = path.hasSuffix("README.md") || path.hasSuffix("HEADER.md")
        let isReportZKPath = path.hasPrefix("/Gui/")
        let isHealthcheck = userAgent.hasPrefix("Xymon")
        let isThemeFile = path.hasPrefix("/Nginx-Fancyindex-Theme-light/") || path == "/favicon.ico"
        let isPermalink = path.hasPrefix("/permalink/")

        return isHtmlReportResource || isDirectoryListingMetadata || isReportZKPath || isHealthcheck || isThemeFile || isPermalink
    }

    var isIrrelevantRequest: Bool {
        let isHighLevelDirectoryListing = ["/", "/phmaven", "/hcob", "/phmaven/", "/hcob/"].contains { String(path) == $0 }
        let isReportManagerLogsFile = path.hasPrefix("/logs")
        let isProbingRequest = userAgent.contains("BA-Browser") || userAgent == "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15" || userAgent == "okhttp/4.9.0"

        return isHighLevelDirectoryListing || isReportManagerLogsFile || isProbingRequest
    }

    private var pathMatch: SimpleRegexMatch? {
        let pattern = "\\/(phmaven|hcob)(?:\\/(\\d+)(?:\\/([^\\/]+)(?:\\/(.*)))?)?"
        return SimpleRegexMatcher.firstMatch(forPattern: pattern, in: String(path))
    }

    var repository: String? {
        return pathMatch?.groups[1].map { String($0) }
    }

    var pipeline: Int64? {
        guard let match = pathMatch, match.groups.count > 2 else { return nil }
        return pathMatch?.groups[2].flatMap { Int64($0) }
    }

    var job: String? {
        guard let match = pathMatch, match.groups.count > 3 else { return nil }
        return pathMatch?.groups[3].map { String($0) }
    }

    var file: String? {
        guard let match = pathMatch, match.groups.count > 4 else { return nil }
        return pathMatch?.groups[4].map { String($0) }
    }
}

extension LogEntry {
    init(from line: String) throws {
        guard let match: SimpleRegexMatch = SimpleRegexMatcher.firstMatch(forPattern: combinedLogPattern, in: line.trimmingCharacters(in: .whitespacesAndNewlines)) else {
//            fatalError("Failed to parse line: \(line)")
            throw Error.lineNotMatching
        }

        self.init(ip: match.groups[1]!, remoteUser: match.groups[2]!, rawTimestamp: match.groups[3]!, method: match.groups[4]!, path: clean(match.groups[5]!), proto: match.groups[6]!, rawStatus: match.groups[7]!, rawBytes: match.groups[8]!, referee: clean(match.groups[9]!), userAgent: match.groups[10]!)
    }
}
