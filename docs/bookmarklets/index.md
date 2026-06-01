---
layout: default
title: Desmos Bookmarklets
permalink: /bookmarklets
has_children: false
---

{: .no_toc }
# {{page.title}}

{: .no_toc .text-delta }
## {{site.toc_header}}

- TOC
{:toc}

There are some features that Desmos is missing that are nice to have when doing external development work. These
features only really require a few lines of Javascript code, and they can be easily added via
[bookmarklets](https://wikipedia.org/wiki/Bookmarklet). The bookmarklet URLs are provided below along with their
recommended bookmark names.

To add a bookmarklet, copy it using the clipboard icon in the corner of the box and add a bookmark manually in your
browser, pasting the code in the field for the bookmark URL.

{: .note }
Bookmarklets copied from the internet can be dangerous since they run arbitrary Javascript code on the current webpage.
As such, some browsers will strip the `javascript:` prefix from the bookmarklet when it is pasted as a bookmark URL (and
perhaps rightfully so). **If this is the case for you, make sure to re-add the prefix manually.** As for whether these
particular bookmarklets are safe? I can assure you that they are, but don't just take my word for it—you can view the
source code yourself by going to the Javascript console in your browser (usually <kbd>F12</kbd>, then the Console tab),
typing `decodeURIComponent("")`, pasting the bookmarklet code between the quotes, and pressing <kbd>Enter</kbd>.

To use a bookmarklet, go to the page you want it to run on, then click the corresponding bookmark.

## Handling Graph Data

### Import graph...

Prompts the user to import a `.json` file representing a graph (this is the format output by the compiler). The file is
loaded into the current graph.

{: .warning }
This does not create a new graph! It is not recommended to use this bookmarklet while a saved graph is open, or else you
may accidentally overwrite the save with the imported graph.

{: #import-graph-url .no_toc }
#### Bookmarklet URL

```
javascript:(()%3D%3E%7Blet%20i%3Ddocument.createElement(%22input%22)%3Bi.type%3D%22file%22%3Bi.accept%3D%22application%2Fjson%22%3Bi.addEventListener(%22change%22%2C()%3D%3E%7Bif(i.files)%7Blet%20f%3Dnew%20FileReader()%3Bf.onerror%3Dalert%3Bf.onload%3D()%3D%3E%7Btry%7BCalc.setState(JSON.parse(f.result))%7Dcatch(e)%7Balert(e)%7D%7D%3Bf.readAsText(i.files%5B0%5D)%7D%7D)%3Bi.click()%7D)()%3B
```

### Export graph...

Prompts the user to export a `.json` file representing the current graph being edited. This is not quite as useful as
importing a graph for the purposes of Desmosify, but it might come in handy.

{: #export-graph-url .no_toc }
#### Bookmarklet URL

```
javascript:(()%3D%3E%7Bif(!Calc)return%20alert(%22This%20action%20is%20only%20allowed%20on%20a%20Desmos%20page.%22)%3Blet%20a%3Ddocument.createElement(%22a%22)%3Ba.href%3D%22data%3Aapplication%2Fjson%3Bcharset%3Dutf-8%2C%22%2BencodeURIComponent(JSON.stringify(Calc.getState()))%3Ba.download%3D(document.getElementById(%22dcg-graph-title-text%22)%3F.textContent%7C%7C%22untitled%22)%2B%22.json%22%3Ba.click()%7D)()%3B
```

### Open graph data in new tab

Opens a new tab with the JSON representation of the current graph being edited. Again, this may not be very useful for
the purposes of Desmosify, but it might come in handy.

{: #open-graph-data-in-new-tab-url .no_toc }
#### Bookmarklet URL

```
javascript:(()%3D%3E%7Bif(!Calc)return%20alert(%22This%20action%20is%20only%20allowed%20on%20a%20Desmos%20page.%22)%3Bwindow.open(%22data%3Aapplication%2Fjson%3Bcharset%3Dutf-8%2C%22%2BencodeURIComponent(JSON.stringify(Calc.getState()))%2C%22_blank%22)%7D)()%3B
```
