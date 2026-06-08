import os
from dataclasses import dataclass

PARTITIONS_CSV = "partitions.csv"
PARTITIONS_RS = os.path.join("cross", "src", "partitions.rs")


@dataclass()
class Partition:
    name: str
    partition_type: str
    sub_type: str
    start: int
    end: int
    flags: str | None = None


def get_partitions(lines: list[str]) -> list[Partition]:
    partitions = []
    for line in lines:
        values = [v.strip() for v in line.split(",")]
        assert len(values) >= 5
        offset = values[3]
        start = int(offset, 16)
        size = values[4]
        assert size.endswith("K")
        size = size.removesuffix("K")
        end = start + int(size) * 1024
        partitions.append(
            Partition(
                name=values[0],
                partition_type=values[1],
                sub_type=values[2],
                start=start,
                end=end,
                flags=values[5] if len(values) > 5 else None,
            )
        )
    return partitions


with open(PARTITIONS_CSV) as f:
    csv_content = f.read()

csv_lines = csv_content.splitlines()

# check if partition table has correct format
assert csv_lines[0] == "# ESP-IDF Partition Table"
assert csv_lines[1] == "# Name, Type, SubType, Offset, Size, Flags"

partitions = get_partitions(csv_lines[2:])

# check if partitions do not overlap and are not bigger than the flash
last_end = int("0x9000", 16)
for partition in partitions:
    assert partition.start == last_end
    last_end = partition.end
assert last_end == 1024 * 4096

with open(PARTITIONS_RS) as f:
    rs_content = f.read()

rs_lines = rs_content.splitlines()

# check if definition ranges match
struct_start = -1
const_start = -1
for index, line in enumerate(rs_lines):
    if "pub struct PartitionTable {" == line.strip():
        struct_start = index + 1
        continue
    if struct_start >= 0 and index < struct_start + len(partitions):
        assert (
            f"pub {partitions[index - struct_start].name}: core::ops::Range<u32>,"
            == line.strip()
        )
    if struct_start > 0 and index == struct_start + len(partitions):
        assert "}" == line.strip()
    if "pub const PARTITIONS: PartitionTable = PartitionTable {" == line.strip():
        const_start = index + 1
        continue
    if const_start >= 0 and index < const_start + len(partitions):
        partition = partitions[index - const_start]
        assert (
            f"{partition.name}: {hex(partition.start)}..{hex(partition.end)},"
            == line.strip()
        )
    if const_start > 0 and index == const_start + len(partitions):
        assert "};" == line.strip()
