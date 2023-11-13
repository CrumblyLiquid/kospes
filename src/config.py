from typing import List, Tuple
from typing_extensions import Self
from pathlib import Path
import json

DEFAULT_PATH: Path = Path("config.json")

class Config:
    channels: List[int]
    courses: List[str]
    seen_events: List[int]

    def __init__(self,
                 channels: List[int],
                 courses: List[str],
                 seen_events: List[int]
                ):
        self.channels = channels
        self.courses = courses
        self.seen_events = seen_events

    def encode_struct(self):
        return {
            "channels": self.channels,
            "courses": self.courses,
            "seen_events": self.seen_events
        }

    @classmethod
    def decode_struct(cls, struct) -> Tuple[List[int], List[str], List[int]]:
        return (struct["channels"], struct["courses"], struct["seen_events"])

    def save(self, path: Path = DEFAULT_PATH):
        with open(path, "w", ) as file:
            json.dump(self.encode_struct(), file, indent = 4)

    @classmethod
    def load(cls, path: Path = DEFAULT_PATH) -> Self:
        with open(path, "r") as file:
            (channels, courses, seen_events) = Config.decode_struct(json.load(file))
            return cls(channels, courses, seen_events)

