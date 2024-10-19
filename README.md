# Strings

- stringsext is to search for multi-byte encoded strings in binary data.
  stringsext is a Unicode enhancement of the GNU strings tool with additional functionalities: stringsext recognizes Cyrillic, Arabic, CJKV characters and other scripts in all supported multi-byte-encodings, while GNU strings fails in finding any of these scripts in UTF-16 and many other encodings.

stringsext prints all graphic character sequences in FILE or stdin that are at least MIN bytes long.

Unlike GNU strings stringsext can be configured to search for valid characters not only in ASCII but also in many other input encodings, e.g.: UTF-8, UTF-16BE, UTF-16LE, BIG5-2003, EUC-JP, KOI8-R and many others. The option --list-encodings shows a list of valid encoding names based on the WHATWG Encoding Standard. When more than one encoding is specified, the scan is performed in different threads simultaneously.

When searching for UTF-16 encoded strings, 96% of all possible two byte sequences, interpreted as UTF-16 code unit, relate directly to Unicode codepoints. As a result, the probability of encountering valid Unicode characters in a random byte stream, interpreted as UTF-16, is also 96%. In order to reduce this big number of false positives, stringsext provides a parametrizable Unicode-block-filter. See --encodings and --same-unicode-block options in the manual page for more details.

stringsext is mainly useful for extracting Unicode content out of non-text files.

When invoked with stringsext -e ascii stringsext can be used as GNU strings replacement.

