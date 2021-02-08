//
//  File.swift
//  
//
//  Created by Til Blechschmidt on 02.01.21.
//

import Foundation
import ArgumentParser

import SQLite

enum SIPrefix: Character {
    case giga = "G"
    case mega = "M"
    case kilo = "K"

    var multiplier: UInt64 {
        switch self {
        case .giga:
            return 1024 * 1024 * 1024
        case .mega:
            return 1024 * 1024
        case .kilo:
            return 1024
        }
    }
}

fileprivate func parse(filesize: Substring) -> UInt64? {
    var filesize = filesize

    if filesize == "0" {
        return 0
    }

    guard let lastChar = filesize.popLast(), let siPrefix = SIPrefix(rawValue: lastChar), let value = Double(filesize) else {
        return nil
    }

    return UInt64(value * Double(siPrefix.multiplier))
}

struct ArtifactPathMatch {
    private static let regExp = "^([0-9]+)\\/logs_([^_]+)_(?:reorg_)?([^_]+)(?:_\\d_\\d)?-([0-9]+)(?:-[^_]+?)?(?:_(SUCCESS|FAILED))?$"

    let pipelineID: Int
    let environment: Substring
    let testSuite: Substring
    let jobID: Int
    let status: Substring?

    var dictionaryKey: ArtifactSizeDictionaryKey {
        ArtifactSizeDictionaryKey(environment: String(environment), testSuite: String(testSuite))
    }

    init?(entry: FileSizeEntry) {
        guard let match = SimpleRegexMatcher.firstMatch(forPattern: ArtifactPathMatch.regExp, in: entry.path), match.groups.count > 4 else {
            return nil
        }

        pipelineID = Int(match.groups[1]!)!
        environment = match.groups[2]!
        testSuite = match.groups[3]!
        jobID = Int(match.groups[4]!)!
        status = match.groups.count > 5 ? match.groups[5] : nil
    }
}

struct FileSizeEntry {
    enum Error: Swift.Error {
        case noSeparatorFound
        case invalidSize
    }

    let size: UInt64
    let path: String

    init(line: String) throws {
        guard let separatorIndex = line.firstIndex(of: "\t") else {
            throw Error.noSeparatorFound
        }

        guard let size = parse(filesize: line[line.startIndex..<separatorIndex]) else {
            print(line[line.startIndex..<separatorIndex])
            throw Error.invalidSize
        }

        self.size = size
        path = line[line.index(after: separatorIndex)..<line.endIndex].trimmingCharacters(in: .newlines)
    }
}

struct ArtifactSizeDictionaryKey: Hashable {
    let environment: String
    let testSuite: String
}

struct PlottableArtifactsData: Encodable {
    let name: String
    var keys: [String] = []
    var values: [UInt64] = []
}

struct GzipListingEntry {
    // Note: This expression only matches paths that start with "./\d+/" since the pipeline folders do contain symlinked directories
    //       Purely for performance and memory usage optimization!
    private static let regExp = "^(\\d+) +(\\d+) +(\\d{1,2}.\\d{1,2})% +\\.\\/(\\d+[^\\n]+)$"

    let uncompressed: UInt64
    let compressed: UInt64
    let path: String

    init?(line: String) {
        guard let match = SimpleRegexMatcher.firstMatch(forPattern: GzipListingEntry.regExp, in: line.trimmingCharacters(in: .whitespacesAndNewlines)),
              match.groups.count > 4,
              let rawCompressed = match.groups[1].flatMap({ UInt64($0) }),
              let rawUncompressed = match.groups[2].flatMap({ UInt64($0) }),
              let rawPath = match.groups[4] else {
            return nil
        }

        uncompressed = rawUncompressed
        compressed = rawCompressed
        path = rawPath.replacingOccurrences(of: "./", with: "")
    }
}

struct Artifacts: ParsableCommand {
    static var configuration = CommandConfiguration(abstract: "Artifact size data")

    @OptionGroup var options: Ingest.Options

    @Option(name: [.long, .customShort("g")], help: "Gzip file listing for size compensation.")
    var gzipListing: String?

    @Argument(help: "JSON output file.")
    var output: String

    func run() throws {
        var compressedFiles: [GzipListingEntry] = []

        if let gzipPath = gzipListing {
            compressedFiles = try process(gzipListing: gzipPath)
        }

        try processInput(compressedFiles)
    }

    func process(gzipListing path: String) throws -> [GzipListingEntry] {
        guard let reader = LineReader(path: path) else {
            throw Ingest.Error.UnableToReadInput
        }

        // Skip the first line which contains the listing header
        _ = reader.nextLine

        var listing: [GzipListingEntry] = []
        var totalCount = 0
        var failedToParse = 0
        for line in reader {
            totalCount += 1
            if let match = GzipListingEntry(line: line) {
                listing.append(match)
            } else {
                failedToParse += 1
            }
        }

        print("Read \(totalCount) GZip listings (\(failedToParse) did not match the path requirements)")

        return listing
    }

    func processInput(_ compressedFiles: [GzipListingEntry]) throws {
        guard let reader = LineReader(path: options.input) else {
            throw Ingest.Error.UnableToReadInput
        }

        var sizeDictionary: [ArtifactSizeDictionaryKey : [UInt64]] = [:]
        var totalCount = 0
        var emptySizes = 0
        var failedToParse = 0

        for line in reader {
            let entry = try FileSizeEntry(line: line)
            totalCount += 1

            // Some jobs have already been cleaned up, ignore them
            if entry.size > 0, let pathMatch = ArtifactPathMatch(entry: entry) {
                let compressedFilesInDirectory = compressedFiles.filter { $0.path.hasPrefix(entry.path) }
                let cumulativeCompressedSize = compressedFilesInDirectory.reduce(0) { $0 + $1.compressed }
                let cumulativeUncompressedSize = compressedFilesInDirectory.reduce(0) { $0 + $1.uncompressed }
                let compensatedSize = entry.size + (cumulativeUncompressedSize - cumulativeCompressedSize)

                sizeDictionary[pathMatch.dictionaryKey, default: []].append(compensatedSize)
            } else if entry.size == 0 {
                emptySizes += 1
            } else {
                failedToParse += 1
                // TODO entry.path.count > 6 && !entry.path.contains("merged_cucumber_html")
            }

            // List non-matching paths
//            if ArtifactPathMatch(entry: entry) == nil && entry.path.count > 6 && !entry.path.contains("merged_cucumber_html") {
//                print(entry.path)
//            }
        }

        print("Read \(totalCount) samples (\(emptySizes) empty, \(failedToParse) non-matching path)")

//        try writeSamples(sizeDictionary)
        try writeJSON(sizeDictionary)
    }

    func writeSamples(_ dict: [ArtifactSizeDictionaryKey : [UInt64]]) throws {
        let db = try Database(path: options.database)

        for (key, values) in dict {
            try autoreleasepool {
                for value in values {
                    try db.db.run(db.jobSizeSamples.table.insert(
                        db.jobSizeSamples.environment <- key.environment,
                        db.jobSizeSamples.testSuite <- key.testSuite,
                        db.jobSizeSamples.bytes <- Int64(value)
                    ))
                }
            }
        }

        // Speed up querying in the simulation :)
        try db.db.execute("""
            CREATE INDEX IF NOT EXISTS "SizeIndex" ON "JobSizeSample" (
                "environment" ASC,
                "testSuite" ASC,
                "id" ASC,
                "bytes" ASC
            )
        """)
    }

    func writeJSON(_ dict: [ArtifactSizeDictionaryKey : [UInt64]]) throws {
        let sortedDict = dict.sorted { (v1, v2) -> Bool in
            let environment = v1.key.environment.compare(v2.key.environment)

            if environment != .orderedSame {
                return environment == .orderedAscending
            } else {
                return v1.key.testSuite.compare(v2.key.testSuite) == .orderedAscending
            }
        }

        let environments = Set(sortedDict.map { $0.key.environment })

        var dataBlocks: [PlottableArtifactsData] = []
        for environment in environments {
            var data = PlottableArtifactsData(name: environment)

            for (key, value) in sortedDict.filter({ $0.key.environment == environment }) {

                // Calculate quartiles, IQR and whiskers to find outlier count
                if value.count >= 100 {
                    let sorted = value.sorted()
                    let q1 = sorted[Int(sorted.count * 1/4)]
                    let q3 = sorted[Int(sorted.count * 3/4)]
                    let iqr = Double(q3 - q1)
                    let lowerWhisker = sorted.first { Double($0) >= (Double(q1) - 1.5 * iqr) }!
                    let upperWhisker = sorted.reversed().first { Double($0) <= (Double(q3) + 1.5 * iqr) }!

                    let lowerOutliers = sorted.filter { $0 < lowerWhisker }
                    let upperOutliers = sorted.filter { $0 > upperWhisker }
                    print("\(environment),\(key.testSuite),\(sorted.count),\(lowerOutliers.count + upperOutliers.count)")
                }

                // This value has been determined using the values from above
                if value.count <= 30 {
//                    let name = ("\(environment) -> \(key.testSuite)").padding(toLength: 50, withPad: " ", startingAt: 0)
//                    print("Skipping\t\(name)\tdue to a low number of samples (\(value.count))")
                    continue
                }
                data.keys.append(contentsOf: Array(repeating: key.testSuite, count: value.count))
                data.values.append(contentsOf: value)
            }

            dataBlocks.append(data)
        }

        let jsonData = try JSONEncoder().encode(dataBlocks)
        try jsonData.write(to: URL(fileURLWithPath: output))
    }
}
