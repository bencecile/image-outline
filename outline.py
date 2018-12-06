"""
Generates an image that is the outline of an input image
"""
import math
from pathlib import Path
import time

from PIL import Image

IMAGE_IN = Path("crystalized.jpg")

WHITE = (255, 255, 255)
BLACK = (0, 0, 0)

def pixelLumin(pixel):
    """
    Figures out the luminance of the pixel
    """
    (red, green, blue) = [i / 255 for i in pixel]
    return 0.2126 * red + 0.7152 * green + 0.0722 * blue

def main():
    # Keep track of a start time
    startTime = time.time_ns()
    def currentTime():
        """
        Returns the current time as a string
        """
        diff = time.time_ns() - startTime
        seconds = diff // 1_000_000_000
        
        # Subtract off any seconds so that they aren't used for everything else
        diff -= seconds * 1_000_000_000
        millis = diff // 1_000_000

        diff -= millis * 1_000_000
        micros = diff // 1_000

        diff -= micros * 1_000
        return f"{seconds}s {millis}ms {micros}μs {diff}ns"

    baseImage = Image.open(IMAGE_IN)
    newImage = Image.new(baseImage.mode, baseImage.size, WHITE)
    print(f"Loaded image ({baseImage.width}x{baseImage.height}): {currentTime()}")

    # Pre-compute the luminance for every pixel so that we won't be doing duplicate work
    # It goes WxH, so it's in vertical rows
    luminMap = [
        [
            pixelLumin(baseImage.getpixel((x, y))) for y in range(baseImage.height)
        ] for x in range(baseImage.width)
    ]
    print(f"Done with the luminance map: {currentTime()}")

    def drawOutline(x, y, compIndices):
        """
        Draws the outline into the new image
        This only places a new pixel at x and y in the new image
        Uses the luminance difference to the other pixels at compIndices

        compIndices is a list of tuples.
        Ex. [(0, 1), (-1, 0)] says to compare 2 pixels. The first one is at (x, y + 1) and the
        second one is at (x - 1, y)
        """
        lumin = luminMap[x][y]
        lumin_sum = 0
        for (dX, dY) in compIndices:
            lumin_sum += abs(lumin - luminMap[x + dX][y + dY])

        averageLumin = lumin_sum / len(compIndices)
        # The sqrt function will make the larger differences more prominent
        grayValue = int((1 - math.sqrt(averageLumin)) * 255)

        newImage.putpixel((x, y), (grayValue, grayValue, grayValue))
    
    # Split up the processing into quarters so that we can start from each corner
    xBounds = [baseImage.width // 2, baseImage.width - 1]
    yBounds = [baseImage.height // 2, baseImage.height - 1]

    # Top left starts at 0, 0 travels +1, +1
    for x in range(xBounds[0]):
        for y in range(yBounds[0]):
            drawOutline(x, y, [(0, 1), (1, 0), (1, 1)])
    print(f"25% Done: {currentTime()}")

    # Bottom left starts at 0, height travels +1, -1
    for x in range(xBounds[0]):
        for y in range(yBounds[1], yBounds[0] - 1, -1):
            drawOutline(x, y, [(0, -1), (1, 0), (1, -1)])
    print(f"50% Done: {currentTime()}")

    # Top right starts at width, 0 travels -1, 1
    for x in range(xBounds[1], xBounds[0] - 1, -1):
        for y in range(yBounds[0]):
            drawOutline(x, y, [(0, 1), (-1, 0), (-1, 1)])
    print(f"75% Done: {currentTime()}")

    # Bottom right starts at width, height travels -1, -1
    for x in range(xBounds[1], xBounds[0] - 1, -1):
        for y in range(yBounds[1], yBounds[0] - 1, -1):
            drawOutline(x, y, [(0, -1), (-1, 0), (-1, -1)])
    print(f"Done: {currentTime()}")

    newImage.save(IMAGE_IN.with_name(f"pyOutline-{IMAGE_IN.stem}{int(time.time())}.png"))
    print(f"Done saving: {currentTime()}")

if __name__ == "__main__":
    main()
