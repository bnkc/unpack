from typing import (
    no_type_check,
    TypeVar,
    Iterable,
    Tuple,
)

import ast


@no_type_check
def foo(bar: str) -> None:
    return 3


# if __name__ == "__main__":
#     print(foo("lev"))


Url = str


def retry(url: Url, retry_count: int) -> None:
    ...


T = TypeVar("T", int, float, complex)
A = TypeVar("A", str, bytes)


def repeat(x: T, n: int) -> list[T]:
    """Return a list containing n references to x."""
    return [x] * n


def longest(x: A, y: A) -> A:
    """Return the longest of two strings."""
    return x if len(x) >= len(y) else y


Vector = Iterable[Tuple[T, T]]


def inproduct(v: Vector[T]) -> T:
    return sum(x * y for x, y in v)


def dilate(v: Vector[T], scale: T) -> Vector[T]:
    return ((x * scale, y * scale) for x, y in v)


vec = []  # type: Vector[float]
