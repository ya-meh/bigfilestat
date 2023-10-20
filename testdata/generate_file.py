#!/usr/bin/python3

import random
import numpy as np

open("bigfile.txt", "w").write(' '.join(list(map(lambda x: str(x), np.random.chisquare(2, 100_000_000)))))