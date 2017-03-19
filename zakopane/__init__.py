import os


version = (0, 0, 1)

NAME = "zakopane"
DEBUG = False


def npj(*args):
    return os.path.normpath(os.path.join(*args))
