//
//  File.swift
//  
//
//  Created by Til Blechschmidt on 15.12.20.
//

import Foundation
import ArgumentParser

struct Ingest: ParsableCommand {
    static var configuration = CommandConfiguration(
        abstract: "Ingest log files.",
        subcommands: [Nginx.self, Gitlab.self, Artifacts.self]
    )
}

extension Ingest {
    struct Options: ParsableArguments {
        @Argument(help: "File to ingest.")
        var input: String

        @Option(name: [.long, .customShort("d")], help: "Database file location.")
        var database: String = "./data.db"
    }

    enum Error: Swift.Error {
        case UnableToReadInput
    }
}
