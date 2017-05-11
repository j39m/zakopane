
import datetime
import os
import zakopane
import zakopane.file
import zakopane.hash

def main(*args):
    if "POSIX_ME_HARDER" in os.environ:
        raise OSError("POSIXing you harder.")

    assert args, "Needs an argument."
    scanDir = args[0]

    return 0
