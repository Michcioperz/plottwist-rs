PlotTwist is a theoretically commandline but not really client for twist.moe pirate anime site.

This is not an endorsement of piracy, it is merely a way for me to access anime unavailable legally in my country.

PlotTwist can be in theory as a library to fetch links to anime from the site.

PlotTwist also contains two binaries, `plottwist-episodes` and `plottwist-series`.

`plottwist-series` fetches a list of anime available on the website and presents it in the form of an ncurses list, with the option to quickly launch `plottwist-episodes` in a separate terminal window (`urxvt` by default, configurable at compile time).

`plottwist-episodes` takes one (1) commandline argument which is the slug/codename (`twist.moe/a/this-part-here`) of a show you want to see, fetches a list of episodes and presents it in the form of an ncurses list, with the option to quickly launch a video player (`mpv` by default, configurable at compile time).

The reason it takes two windows to open two lists is because the whole thing is written in Rust and I could not be bothered to figure lifetimes out properly, and thus I bodged.

PlotTwist was initially developed by me under the name of `qieyia` but I lost access to the email account behind that identity.
