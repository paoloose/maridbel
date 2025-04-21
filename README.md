# Maridbel

[/ˈma.ɾið.bel/](https://ipa-reader.com/?text=%2F%CB%88ma.%C9%BEi%C3%B0.bel%2F)

Maridbel (from my mom's name Maribel) or MariDB is a disk-oriented OLTP Database
Management System (DBMS) written in Rust.

It is designed to be a simple and efficient database engine for educational
purposes, inspired by design philosophies from the CMU 15-445/645 course.

I'll remove this line of text until this project is in a usable state.

## Acknowledgements

- CMU 15-445/645 - <https://15445.courses.cs.cmu.edu/fall2024/>
- German strings - <https://cedardb.com/blog/german_strings>

## Pending optimizations

- Index prefetching (CMU #06)
- Are there any compression mechanisms for row oriented storage? (CMU #05)
- Scan sharing: multiple queries attached to the same cursor (CMU #06)
