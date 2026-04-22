This is the scanner for a music library.

Data structure assumptions:

Disk data:

- Mastering (master recording): an actual music file on disk
    - Has one Recording ID, plus hints for other MB entities to look up in the library
    - May have precalculated ReplayGain data and AccoustID data
    - May have embedded art
    - May have embedded lyrics
    - May have on-disk lyrics
    - We ignore all other data in this file right now, aside from in fallback cases
- Folder
    - May have a cover file, which should be used for releases hinted in the folder.
    - Otherwise ignored

Remote data:

See https://musicbrainz.org/doc/MusicBrainz_Identifier

A Recording ID may have multiple Masterings (particularly due to federation).

We associate Lyrics from disk with Masterings, as this is where they are found on disk. However, in proper Lyrics should be associated with Works. For this reason, we should look up lyrics in this order: Mastering, (Track), Recording, Work. Ideally, all possible matches should be presented to the user in case of the match being undesirable.

Similarly, album art is properly matched with a Release. However:

1. When we read on disk, we get embedded art as well as folder art (folder art being harder to correctly match)
2. Oftentimes the cover art for a single is more interesting and unique than the cover art for an album or EP

For this reason, a more complicated lookup logic should be used, although this may depend more on the preference of the user.
