version = (0, 0, 1)

DEBUG = False


import os
from zakopane.file import SumFile, newSumFile
from zakopane.hash import readHashLine, doHashFile, formatHashLine

def main():
    if "POSIX_ME_HARDER" in os.environ:
        raise OSError("POSIXing you harder.")

    return 0
