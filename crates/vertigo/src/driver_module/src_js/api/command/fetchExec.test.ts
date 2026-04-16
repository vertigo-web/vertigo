import { parseJsonBody } from "./fetchExec";

function assert(condition: boolean, message: string) {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
}

function testEmptyBodyReturnsNull() {
    const result = parseJsonBody("");
    assert(result === null, "empty body returns null");
}

function testJsonBodyIsParsed() {
    const result = parseJsonBody('{"key":"value"}');
    assert(
        typeof result === "object" && result !== null && (result as any)["key"] === "value",
        "non-empty body is parsed as JSON"
    );
}

console.log("\n--- Test parseJsonBody ---");

testEmptyBodyReturnsNull();
testJsonBodyIsParsed();
