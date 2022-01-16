WIP

```sql
select (ip >> 24) || '.' || ((ip >> 16) & 255) || '.' || ((ip >> 8) & 255) || '.' || (ip & 255) as ipstr, * from _turbonet_peers;
```
