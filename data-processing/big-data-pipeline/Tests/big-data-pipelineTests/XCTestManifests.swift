import XCTest

#if !canImport(ObjectiveC)
public func allTests() -> [XCTestCaseEntry] {
    return [
        testCase(big_data_pipelineTests.allTests),
    ]
}
#endif
