#!/usr/bin/env python3
# -*- coding: UTF-8 -*-
"""Test the program with the given image."""
from pathlib import Path
import subprocess
from PIL import Image
import numpy as np


IMAGE = Path("motion_photo.jpg")


def main() -> None:
    output_image_path = IMAGE.with_suffix(".image.jpg")
    output_debug_path = IMAGE.with_suffix(".debug.bin")
    output_video_path = IMAGE.with_suffix(".motion.mp4")

    # Unlink if they already exist
    output_image_path.unlink(True)
    output_debug_path.unlink(True)
    output_video_path.unlink(True)

    # Make sure that the test files were generated
    assert not output_image_path.is_file()
    assert not output_debug_path.is_file()
    assert not output_video_path.is_file()

    # Run the tool
    subprocess.run(["cargo", "run", "motion_photo.jpg", "-dim"])

    original_image = Image.open(IMAGE)
    output_image = Image.open(output_image_path)

    assert (np.asarray(original_image) == np.asarray(output_image)).all()
    assert original_image.getexif() == output_image.getexif()

    # TODO: Check each image segment, but with special checks for XMP?
    # TODO: make sure XMP of new is valid XML
    # TODO: Make sure extended XMP is the same


if __name__ == "__main__":
    main()
