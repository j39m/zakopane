
import zakopane
import hashlib


hasher = hashlib.sha512
HASHLEN = len(hasher().hexdigest())
HASHSEP = " "
FREADTO = (-HASHLEN - len(HASHSEP))

def readHashLine(line):
    """
    Given a line formatted exactly as "<filename> <hash>" (single space as
    separator, NO trailing or leading characters - especially not
    whitespace), return the filename and hash together as a tuple.
    """
    return (line[:FREADTO], line[-HASHLEN:])


def doHashFile(fname):
    """
    Given a path to a file, return its checksum.
    """
    with open(fname, "rb") as fObj:
        hObj = hasher()
        hObj.update(fObj.read())
        hash_ = hObj.hexdigest()
    return hash_


def formatHashLine(fname):
    hash_ = doHashFile(fname)
    return HASHSEP.join((fname, hash_))
