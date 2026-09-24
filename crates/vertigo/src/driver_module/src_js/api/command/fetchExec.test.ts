import { decodeBody, parseJsonBody } from "./fetchExec";

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

const textOf = (content: ReturnType<typeof decodeBody>): string | undefined =>
    "Text" in content ? content.Text : undefined;

function testDecodeBodyParsesJson() {
    const result = decodeBody("application/json", '{"key":"value"}');
    assert(
        "Json" in result && (result.Json as any)["key"] === "value",
        "JSON body is decoded as Json"
    );
}

function testDecodeBodyFallsBackToText() {
    assert(textOf(decodeBody(null, "OK")) === "OK", "non-JSON body without Content-Type falls back to Text");
    assert(
        textOf(decodeBody("text/html", "<html>Bad Gateway</html>")) === "<html>Bad Gateway</html>",
        "non-JSON html body falls back to Text"
    );
}

function testDecodeBodyTextPlainIsNeverJson() {
    assert(textOf(decodeBody("text/plain", "true")) === "true", "bare text/plain stays Text");
    assert(textOf(decodeBody("text/plain; charset=utf-8", "123")) === "123", "text/plain with charset stays Text");
    assert(textOf(decodeBody("Text/Plain;charset=utf-8", "")) === "", "text/plain match is case-insensitive");
}

function testDecodeBodyEmptyIsNull() {
    const result = decodeBody(null, "");
    assert("Json" in result && result.Json === null, "empty body is decoded as Json null");
}

console.log("\n--- Test parseJsonBody ---");

testEmptyBodyReturnsNull();
testJsonBodyIsParsed();

console.log("\n--- Test decodeBody ---");

testDecodeBodyParsesJson();
testDecodeBodyFallsBackToText();
testDecodeBodyTextPlainIsNeverJson();
testDecodeBodyEmptyIsNull();
