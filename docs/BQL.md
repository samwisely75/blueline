# Blueline Query Language (BQL)

Blueline Query Language (BQL) is an IMAGINARY, domain-specific language designed for querying and manipulating data within the blueline. The language syntax is a blend of HTTP, JSON Query Language (JQL), XML Query Language (XQL), Cascade Style Sheet (CSS) Selector, Regular Expression (Regex), and SQL. It is designed for real hard-core black-and-white console jankies who want to play with waves of text data in the terminal like Matrix operators in the 21st century.

## Example

```
GET /_cat/shards
| SORT node, index, prirep, shard
| SELECT node, index, shard, prirep
| TO CSV
| SAVE ~/shards.csv
```

```
GET /employees
| SELECT .employees[]
| WHERE .age > 30 AND .employ_status == "active"
| POST @profileB/expo/register
WITH BODY IN FORM
    name: .name
    email: .email
```

```
POST /_query
| WITH YAML
  query:
    bool:
      must:
        - match:
            title: "Blueline"
        - range:
            @timestamp:
              gte: "2023-01-01"
              lte: "${now/d}"
  size: 10
  sort:
    - @timestamp: desc
| JSON
```

## Syntax