"""What is detectable, and what each answer carries.

:class:`FileKind` is a closed enum with one member per format in the crate's
table, so iterating it is the authoritative list of what this build recognises.

Run it with::

    python 04_list_supported_formats.py
"""

from __future__ import annotations

from magical_py import FileKind


def by_media_type() -> dict[str, int]:
    """Count formats per top-level media type, most common first."""
    tally: dict[str, int] = {}
    for kind in FileKind:
        mime = kind.mime
        if mime is None:
            continue
        name = mime.split("/")[0]
        tally[name] = tally.get(name, 0) + 1
    return dict(sorted(tally.items(), key=lambda item: (-item[1], item[0])))


def without(attribute: str) -> list[str]:
    """Return the names of the members whose *attribute* is ``None``."""
    return sorted(kind.name for kind in FileKind if getattr(kind, attribute) is None)


def main() -> None:
    print(f"{len(FileKind)} formats are detectable by this build.")

    print()
    print("By top-level media type:")
    for name, count in by_media_type().items():
        print(f"  {name}: {count}")

    no_mime = without("mime")
    print()
    print(f"{len(no_mime)} formats have no registered media type, which is None rather")
    print("than a guess:")
    print("  " + ", ".join(no_mime))

    no_extension = without("extension")
    print()
    print(f"{len(no_extension)} formats have no conventional extension:")
    print("  " + ", ".join(no_extension))

    # A member answers to two names: the Rust variant name, and the crate's own
    # short value. Either one comes back to the same member.
    print()
    print("A member can be looked up by either of its names:")
    print(f"  FileKind.Png.value: {FileKind.Png.value!r}")
    print(f"  FileKind.from_value({FileKind.Png.value!r}): {FileKind.from_value('png').name}")
    print(f"  FileKind[{FileKind.Png.name!r}]: {FileKind['Png'].name}")


if __name__ == "__main__":
    main()
