"""Check developer task IDs and source-to-spec references, not protocol correctness."""
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent / 'docs'))
from check_spec import heading_ids


def main():
    root = Path(__file__).resolve().parents[1]
    guide = (root / 'docs/03-reference/07-development-order.md').read_text()
    task_ids = set(re.findall(r'<a id="([a-z]+-\d{2})"', guide))
    failures = []
    references = todos = 0
    for source in sorted(root.glob('*/src/**/*.rs')):
        crate = next(parent for parent in source.parents if (parent / 'Cargo.toml').is_file())
        text = source.read_text()
        for target in re.findall(r'Spec: ([^\s`]+)', text):
            references += 1
            name, _, fragment = target.partition('#')
            document = crate / name
            if not document.is_file():
                failures.append(f'{source.name}: missing spec {target}')
            elif fragment:
                body = document.read_text()
                anchors = heading_ids(body) | set(re.findall(r'<a id="([^"]+)"', body))
                if fragment not in anchors:
                    failures.append(f'{source.name}: missing spec anchor {target}')
        for marker in re.finditer(r'^\s*todo!\(', text, re.M):
            todos += 1
            message = re.match(r'\s*"([A-Z]+-\d{2}):', text[marker.end():])
            if not message or message[1].lower() not in task_ids:
                failures.append(f'{source.name}: TODO lacks a documented task ID')
            preceding = '\n'.join(text[:marker.start()].splitlines()[-8:])
            if not re.search(r'// Spec: README\.md#', preceding):
                failures.append(f'{source.name}: TODO lacks a nearby Spec comment')
    # The root remains a usable entry point after moving the full research tree.
    for source in (root / 'README.md', root / 'AGENTS.md', *root.glob('*/README.md')):
        for target in re.findall(r'\]\(([^\s)]+)\)', source.read_text()):
            if re.match(r'[a-z][a-z0-9+.-]*:', target, re.I):
                continue
            name, _, fragment = target.partition('#')
            destination = source.parent / name if name else source
            if not destination.exists():
                failures.append(f'{source.name}: missing link {target}')
            elif fragment and destination.suffix == '.md':
                body = destination.read_text()
                if fragment not in heading_ids(body) | set(re.findall(r'<a id="([^"]+)"', body)):
                    failures.append(f'{source.name}: missing anchor {target}')
    for failure in failures:
        print(failure, file=sys.stderr)
    print(f'{todos} TODO operations, {references} spec references, {len(task_ids)} assignments; {len(failures)} errors')
    return bool(failures)


if __name__ == '__main__':
    raise SystemExit(main())
