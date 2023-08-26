#!/usr/bin/env python3
# -*- coding: UTF-8 -*-
"""Compare the bytes of the images to the length of the smaller image."""
from argparse import ArgumentParser
from pathlib import Path
from typing import cast


def main() -> None:
    """Compare the images."""

    parser = ArgumentParser(description=__doc__)
    parser.add_argument("image1", type=Path, help="First image to compare")
    parser.add_argument("image2", type=Path, help="Second image to compare")
    arguments = parser.parse_args()

    image1 = cast(Path, arguments.image1).read_bytes()
    image2 = cast(Path, arguments.image2).read_bytes()
    if len(image1) < len(image2):
        compare_len = len(image1)
        assert image1 == image2[:compare_len]
    else:
        compare_len = len(image2)
        assert image1[:compare_len] == image2


if __name__ == "__main__":
    main()
