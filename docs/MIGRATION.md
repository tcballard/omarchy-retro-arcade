# Source provenance

The original five source trees were imported with their complete Git ancestry.
2048 preserves its complete original Git history in `games/2048/upstream-history.bundle`;
the connector-published PR does not attach those commits as ancestry. The source repositories are retained until the unified app is accepted.

| Game | Source | Imported commit |
|---|---|---|
| chess | https://github.com/tcballard/omarchy-chess.git | 1a12a8fa6e6e31578ca603d2f28dbbf3b9ccc8fe |
| solitaire | https://github.com/tcballard/omarchy-solitaire.git | 719865898e71e3b4cfad3a23da4709e735edd5b3 |
| scram | https://github.com/tcballard/omarchy-scram.git | c7819a95061734b78b77091efb04251231767d4d |
| invaders | https://github.com/tcballard/omarchy-invaders.git | 6e79fd5b528c98c8710c36d875d1f14d40daa171 |
| pinball | https://github.com/tcballard/omarchy-spacecadet.git | ff47f9c70107fe0cc8eb750f7d3e8917929426d3 |
| 2048 | https://github.com/avibarit/2048.git | 3f10becf38aa40a21c1555091f677c272830788a |

2048 preserves all four upstream commits, including Avi Barit’s original authorship.
The unchanged source snapshot is in `games/2048/upstream/`; the native port is
in `games/2048/src/`. See [credits and licence evidence](../games/2048/THIRD_PARTY.md).
