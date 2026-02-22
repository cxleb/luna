#! /bin/sh

cargo test

cargo build

# Test success cases - these should pass
echo "=== Testing Success Cases ==="
success_count=0
total_success=0
for filename in tests/success/*; do 
    if [ -f "$filename" ]; then
        total_success=$((total_success + 1))
        echo "Running success test: $filename";
        if ./target/debug/luna-rs "$filename" >/dev/null 2>&1; then
            echo "✓ PASSED: $filename"
            success_count=$((success_count + 1))
        else
            echo "✗ FAILED: $filename (expected to pass)"
        fi
        echo ""
    fi
done

echo "Success Tests: $success_count/$total_success passed"
echo ""

# Test failure cases - these should fail
echo "=== Testing Failure Cases ==="
failure_count=0
total_failure=0
for filename in tests/failure/*; do 
    if [ -f "$filename" ]; then
        total_failure=$((total_failure + 1))
        echo "Running failure test: $filename";
        if ! ./target/debug/luna-rs "$filename" >/dev/null 2>&1; then
            echo "✓ PASSED: $filename (correctly failed)"
            failure_count=$((failure_count + 1))
        else
            echo "✗ FAILED: $filename (should have failed but passed)"
        fi
        echo ""
    fi
done

echo "Failure Tests: $failure_count/$total_failure passed"
echo ""

# Summary
total_tests=$((total_success + total_failure))
total_passed=$((success_count + failure_count))

echo "=== Test Summary ==="
echo "Success Tests: $success_count/$total_success passed"
echo "Failure Tests: $failure_count/$total_failure passed"
echo "Total: $total_passed/$total_tests tests passed"

if [ $total_passed -eq $total_tests ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "❌ Some tests failed!"
    exit 1
fi