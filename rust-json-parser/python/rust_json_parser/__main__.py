import os.path
import sys

from rust_json_parser import benchmark_performance, dumps, parse_json, parse_json_file


def generate_json(size: str) -> str:
    if size == "small":
        return '{"key": "value"}'
    if size == "medium":
        items = ",".join(f'{{"id": {i}, "name": "item_{i}"}}' for i in range(100))
        return f"[{items}]"
    items = ",".join(
        f'{{"id": {i}, "name": "item_{i}", "active": true}}' for i in range(1000)
    )
    return f"[{items}]"


def run_benchmark(label: str, json_str: str) -> None:
    rust_time, json_time, simplejson_time = benchmark_performance(json_str)

    json_speedup = json_time / rust_time
    simplejson_speedup = simplejson_time / rust_time

    print(f"\n{label} ({len(json_str)} bytes):")
    print(f"  Rust:            {rust_time:.6f}s")
    print(
        f"  Python json (C): {json_time:.6f}s "
        f"(Rust is {json_speedup:.2f}x {'faster' if json_speedup > 1 else 'slower'})"
    )
    print(
        f"  simplejson:      {simplejson_time:.6f}s "
        f"(Rust is {simplejson_speedup:.2f}x {'faster' if simplejson_speedup > 1 else 'slower'})"
    )


if __name__ == "__main__":
    if "--benchmark" in sys.argv:
        print("Running JSON parser benchmarks...")
        run_benchmark("Small JSON", generate_json("small"))
        run_benchmark("Medium JSON", generate_json("medium"))
        run_benchmark("Large JSON", generate_json("large"))
    else:
        try:
            # No argument - read from stdin
            if len(sys.argv) == 1:
                data = parse_json(sys.stdin.read())
            # Argument is an existing file path
            elif os.path.exists(sys.argv[1]):
                data = parse_json_file(sys.argv[1])
            else:
                data = parse_json(sys.argv[1])

            print(dumps(data, indent=2))

        except ValueError as e:
            print(f"JSON error: {e}", file=sys.stderr)
            sys.exit(1)
        except IOError as e:
            print(f"File error: {e}", file=sys.stderr)
            sys.exit(1)
