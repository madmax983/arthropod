with open("deny.toml", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if '"MIT",' in line:
        lines.insert(i, '    "CDLA-Permissive-2.0",\n')
        break

with open("deny.toml", "w") as f:
    f.writelines(lines)
