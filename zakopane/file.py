"""
Defines the files used for zakopane.
"""


import zakopane

METASEP = "=" * 52
METAKVSEP = ":"


class SumFile(object):
    """
    A representation of the file used for storing off checksum info.
    """

    def __init__(self, fileName, existing=True):
        self._slurp = None

        # If we are reading an existing db, set things up.
        if existing:
            with open(fileName, "r") as fObj:
                slurp = [l.strip() for l in fObj.readlines()]
                if zakopane.DEBUG:
                    self._slurp = slurp

        # Otherwise, we're creating a new db; set things up differently.
        else:
            pass

    @staticmethod
    def readMeta(slurp):
        """
        Reads metadata stored in the checksum file. By fiat I decree that
        this shall all be at the top of the file. It will be opened by
        METASEP (at time of writing this is 52 ``='' signs) and closed by
        the same.

        After that, everything inside the metadata is a series of key-value
        pairs.

        This method returns a tuple: the first index at which non-meta data
        can be read and a dictionary of metadata.
        """
        pickupIndex =   -1
        metaDict =      dict()

        assert slurp, "SumFile shall not be empty."

        for line in slurp:
            pickupIndex += 1

            if not pickupIndex:
                assert line == METASEP, "SumFile shall begin with METASEP."
                continue

            if line == METASEP:
                pickupIndex += 1
                break

            (key, val) = line.split(METAKVSEP)
            assert key not in metaDict,\
                "key ``%s'' shall not exist in the metaDict." % key
            metaDict[key] = val

        return (pickupIndex, metaDict)
