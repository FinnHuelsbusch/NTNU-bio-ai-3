# IT3708 Project 3 - Evaluator

This evaluator tests your ground truth images (white background with black segmentation lines).
You can provide either .jpg, .png or a .txt file. The .txt file should have the same format as given in the file “example of txt file.txt”: each cell contains one of two possible numbers: 0 is black and 255 is white.

## Requirements

- Python 3
- NumPy
- If your segmentation algorithm provides .png or .jpg, you need to install pillow (as PIL is now discontinued). If not, remove the import line of PIL.

Installation via PIP or Conda is the easiest way.

## How to run it

1. Place the benchmark files in the folder `optimal_segments`.
2. Place your own segmentation files in `student_segments`.
3. Run `run.py` (e.g. `python -m run` from a terminal)

NB: Run time is largely affected by the number of black pixels present in your ground truth images, specially since this is a very inefficient code in a mostly inefficient language :^)

