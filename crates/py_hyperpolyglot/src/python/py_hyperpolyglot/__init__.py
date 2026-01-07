"""Python bindings for hyperpolyglot language detection."""

from py_hyperpolyglot._py_hyperpolyglot import detect_languages

__all__ = ["detect_languages", "detect_languages_table"]


def detect_languages_table(table):
    """Detect languages for a PyArrow Table with 'content' column (and optional 'filename')."""
    import pyarrow as pa

    contents = table.column("content").to_pylist()
    filenames = table.column("filename").to_pylist() if "filename" in table.schema.names else [None] * len(contents)

    languages, methods = detect_languages(contents, filenames)

    return table.append_column("language", pa.array(languages)).append_column("detection_method", pa.array(methods))
