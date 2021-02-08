import Foundation
import ArgumentParser

let cutoffDate = Date(timeIntervalSince1970: 1608049887) // 2020-12-15 16:31:26 +0000

struct BigDataPipeline: ParsableCommand {
    static var configuration = CommandConfiguration(
        abstract: "A utility for processing log data regarding CI pipelines and their artifacts.",
        subcommands: [Ingest.self],
        defaultSubcommand: Ingest.self)
}

BigDataPipeline.main()
