# Overview

The solver builds a sparse matrix from the neighbor list, then runs conjugate
gradient until the residual drops below 1e-8. We tested the path on three
systems: bulk copper, a small supercell, and a free surface.

Runtime on the 256-atom cell is about 40 milliseconds per solve on a single
CPU core.
