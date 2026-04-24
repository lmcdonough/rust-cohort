import os.path
import sys

from rust_json_parser import dumps, parse_json, parse_json_file

if __name__ == "__main__":
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