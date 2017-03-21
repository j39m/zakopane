"""
Defines the files used for zakopane.
"""


import zakopane
import datetime
import functools
import json
import os
import uuid
import xdg.BaseDirectory

METASEP =       "=" * 52
METAKVSEP =     ": "

ROOT = "Root"
WHEN = "When"


def newSumFile(froot):
    return SumFile(froot, reading=False)


@functools.total_ordering
class SumFile(object):
    """
    A representation of the file used for storing off checksum info.

    This shall behave like a glorified dictionary.
    """

    def __init__(self, fileName, reading=True, **kwargs):
        self._slurp =           None
        self._meta =            None
        self._sumDict =         None
        self.when =             datetime.datetime.utcnow().timestamp()
        self.root =             fileName

        # If we are reading an existing db, set things up.
        if reading:
            with open(self.root, "r") as fObj:
                slurp = list()
                for line in fObj:
                    slurp.append(line.strip())
                if zakopane.DEBUG:
                    self._slurp = slurp

            (pickupIndex, meta) = self._readMeta(slurp)
            self._meta = meta
            self._getMeta(meta)
            self._sumDict = self._getSums(pickupIndex, slurp)

        # Otherwise, we're creating a new db; set things up differently.
        else:
            raise NotImplementedError("Can't write SumFiles yet.")

    def __getitem__(self, key):
        return self._sumDict[key]

    def __contains__(self, key):
        return key in self._sumDict

    def __lt__(self, other):
        return self.when < other.when

    def __eq__(self, other):
        return self.when == other.when

    @staticmethod
    def _readMeta(slurp):
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

    def _getMeta(self, metaDict):
        """
        Sets the required metadata in this object. Currently, that's only
        the write date and the root at which the checksum walk begins.

        All other metadata in the metaDict is ignored for now.
        """
        self.when = float(metaDict[WHEN])
        self.root = os.path.normpath(metaDict[ROOT])

    def _getSums(self, pickupIndex, slurp):
        """
        Given a total slurp (including metadata at the head) and a pickup
        index, read in everything at and after the pickup index and parse
        it as a series of checksumming specs. Return the result as a dict.
        """
        sumDict = dict()
        i = pickupIndex

        while i < len(slurp):
            line = slurp[i]
            (fname, fhash) = zakopane.readHashLine(line)
            self._sumDict[fname] = fhash
            i += 1


class DbMapFile(object):
    """
    Represents the database map file we use.
    Currently, its sole purpose is to map configured digest paths to the
    individual SumFile names (or at least their prefixes).
    """
    ddbDir = zakopane.npj(
        xdg.BaseDirectory.xdg_data_home,
        zakopane.NAME,
    )
    cfgDir = zakopane.npj(
        xdg.BaseDirectory.xdg_config_home,
        zakopane.NAME,
    )

    cfgFname = zakopane.npj(
        cfgDir,
        "zakopane.conf",
    )
    dbMapFname = zakopane.npj(
        ddbDir,
        "ddbmap",
    )
    def __init__(self):
        self.dbMap = None
        self.dbValues = None
        for d in (self.ddbDir, self.cfgDir):
            if not os.path.isdir(d):
                os.mkdir(d)

        try:
            with open(self.dbMapFname, "r") as dbMapFile:
                self.dbMap = json.load(dbMapFile)
                self.dbValues = set(self.dbMap.values())
        except (IOError, OSError):
            self.dbMap = dict()
            self.dbValues = set()

    def __getitem__(self, key):
        return self.dbMap[key]

    def __contains__(self, key):
        return key in self.dbMap

    def __setitem__(self, key, value):
        if key in self.dbMap:
            raise KeyError("key %s already in dbMap!" % key)
        if value in self.dbValues:
            raise KeyError("value %s already in dbMap!" % value)
        self.dbMap[key] = value
        self.dbValues.add(value)

    def dryAddKeyValue(self, key, value):
        """
        Returns 0 iff key not in dbMap and value not mapped to.
        Use this to verify that you can safely add the key/value pair.
        Alternatively, call add(key) for guaranteed (???) safety.
        """
        rv = 0
        if key in self.dbMap:
            rv += 1
        if value in self.dbValues:
            rv += 2
        return rv

    def add(self, key):
        """Use a uuid to provide safety in adding."""
        value = str(uuid.uuid4())
        self.__setitem__(key, value)

    def commit(self):
        with open(self.dbMapFname, "w") as dbMapFile:
            json.dump(self.dbMap, dbMapFile)
        return 0

