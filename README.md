# mcping
Ping program for Minecraft.

This program lets you ping minecraft servers to see latency, or see the server IPv4 address if you wish.

## **Features:**
1. SRV record support
2. Supports Server List Ping from 1.7 to 1.17.1
3. Shows all addresses on a domain (if the server is using round-robin DNS)

## Example:
Let's ping the popular server `mc.hypixel.net`.
```
$ ./mcping mc.hypixel.net
mcping - no protocol version provided, using default -1
1: 172.65.195.110
2: 172.65.230.98
3: 172.65.234.205
4: 172.65.216.245
5: 172.65.208.218
6: 172.65.196.232
7: 172.65.210.190
8: 172.65.246.198
9: 172.65.254.166
10: 172.65.211.101
11: 172.65.217.91
12: 172.65.241.114
13: 172.65.223.54
14: 172.65.236.36
15: 172.65.206.176
16: 172.65.230.166
17: 172.65.238.120
18: 172.65.229.74
19: 172.65.245.94
20: 172.65.213.70
mcping - multiple addresses found. which do we use? (0 for all)
```
We see here that Hypixel has many proxy servers. (Not sure how the Minecraft client behaves, probably uses the first one)
We'll select number 1.
```
...
mcping - multiple addresses found. which do we use? (0 for all)
1
mcping - attempting to ping 172.65.195.110:25565...
mcping - server description:
"             §aHypixel Network  §c[1.8-1.17]\n          §5§lSKYBLOCK CRYSTAL HOLLOWS!"
mcping - server version:
   --- "Requires MC 1.8 / 1.17"
   --- protocol version 47
mcping - players:
   --- 55887/200000
mcping - [172.65.195.110:25565] ping: 69420ms
```
And this returns server list information. If you run with the `--ping` argument however:
```
$ ./mcping --ping mc.hypixel.net
mcping - no protocol version provided, using default -1
1: 172.65.230.98
...
20: 172.65.254.166
mcping - multiple addresses found. which do we use? (0 for all)
1
mcping - attempting to ping 172.65.230.98:25565...
mcping - [172.65.230.98:25565] ping: 69420ms
```
It only shows latency. If you run with `--ping` and specify to check all addresses:
```
$ ./mcping --ping mc.hypixel.net
mcping - no protocol version provided, using default -1
1: 172.65.216.245
...
20: 172.65.229.74
mcping - multiple addresses found. which do we use? (0 for all)
0
mcping - testing all addresses...
mcping - [172.65.216.245:25565] ping: 420ms
mcping - [172.65.245.94:25565] ping: 420ms
mcping - [172.65.217.91:25565] ping: 420ms
mcping - [172.65.230.166:25565] ping: 420ms
mcping - [172.65.230.98:25565] ping: 420ms
mcping - [172.65.206.176:25565] ping: 420ms
mcping - [172.65.196.232:25565] ping: 420ms
mcping - [172.65.238.120:25565] ping: 420ms
mcping - [172.65.241.114:25565] ping: 420ms
mcping - [172.65.213.70:25565] ping: 420ms
mcping - [172.65.236.36:25565] ping: 420ms
mcping - [172.65.211.101:25565] ping: 420ms
mcping - [172.65.246.198:25565] ping: 69ms
mcping - [172.65.254.166:25565] ping: 420ms
mcping - [172.65.223.54:25565] ping: 420ms
mcping - [172.65.210.190:25565] ping: 420ms
mcping - [172.65.234.205:25565] ping: 420ms
mcping - [172.65.208.218:25565] ping: 420ms
mcping - [172.65.195.110:25565] ping: 420ms
mcping - [172.65.229.74:25565] ping: 420ms
mcping - best ping is 172.65.246.198:25565 with a time of 69ms
```
## Example #2: 
Let's try a server that uses SRV records. For example, Mineplex at `mineplex.com`.
```
$ ./mcping mineplex.com
mcping - got SRV record for us.mineplex.com.:25565, using it instead
mcping - no protocol version provided, using default -1
1: 173.236.67.28
2: 173.236.67.25
3: 173.236.67.37
4: 173.236.67.24
5: 173.236.67.30
6: 173.236.67.13
7: 173.236.67.20
8: 173.236.67.15
9: 173.236.67.29
10: 173.236.67.31
mcping - multiple addresses found. which do we use? (0 for all)
```
The program finds the SRV record and routes us to the correct location.
