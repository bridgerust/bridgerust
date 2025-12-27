import sys
import asyncio
from embex import cli_main

def main():
    # Pass arguments to Rust.
    # Rust expects [program_name, arg1, arg2...]
    # sys.argv provides exactly that.
    
    try:
        loop = asyncio.get_event_loop()
        loop.run_until_complete(cli_main(sys.argv))
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
