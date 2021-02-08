//
//  File.swift
//  
//
//  Created by Til Blechschmidt on 15.12.20.
//

import Foundation

/// Read text file line by line efficiently
public class LineReader {
    public let path: String
    public let size: Int

    fileprivate let file: UnsafeMutablePointer<FILE>!

    init?(path: String) {
        self.path = path
        file = fopen(path, "r")
        guard file != nil else { return nil }

        fseek(file, 0, SEEK_END)
        size = ftell(file)
        rewind(file)
    }

    public var nextLine: String? {
        var line: UnsafeMutablePointer<CChar>? = nil
        var linecap: Int = 0
        defer { free(line) }
        return getline(&line, &linecap, file) > 0 ? String(cString: line!) : nil
    }

    public var position: Int {
        ftell(file)
    }

    deinit {
        fclose(file)
    }
}

extension LineReader: Sequence {
    public func makeIterator() -> AnyIterator<String> {
        return AnyIterator<String> {
            return self.nextLine
        }
    }
}
