import os
import zakopane
import zakopane.file
import zakopane.hash

def main():
    if "POSIX_ME_HARDER" in os.environ:
        raise OSError("POSIXing you harder.")

    return 0
