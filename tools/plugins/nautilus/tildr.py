#!/usr/bin/env python3
# Maintained in: (c) 2026 OrbitBits. All rights reserved.
#
# The MIT License (MIT)
# Copyright (c) 2026 William C. Canin <https://williamcanin.github.io>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:

# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.

# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
# THE SOFTWARE.

import subprocess
import os
import urllib.parse
import gi
gi.require_version('Nautilus', '4.1')
from gi.repository import Nautilus, GObject

# HOME User
HOME = os.path.expanduser("~")

# Note: This PATH must be the same as the one the '.sh' installer
# (tools/installers/linux.sh) uses to install Tildr.
TILDR_PATH = "tildr"


def get_rel(f):
    uri = f.get_uri()
    path = urllib.parse.unquote(uri.replace("file://", "")).rstrip("/")
    return path.replace(HOME + "/", "")


class TildrMenuProvider(GObject.GObject, Nautilus.MenuProvider):
    def __init__(self):
        super().__init__()
        self._items = []

    def get_file_items(self, files):
        # type 1 = file or symlink, type 2 = folder
        targets = [f for f in files if f.get_file_type() == 1]
        if not targets:
            return []

        rels = [get_rel(f) for f in targets]

        root = Nautilus.MenuItem(
            name="TildrExtension::tildr",
            label="Tildr",
            tip="Manage with Tildr",
        )
        submenu = Nautilus.Menu()
        root.set_submenu(submenu)

        commands = [
            ("add",     "Add File"),
            ("unlink",  "Unlink File"),
            ("restore", "Restore File"),
        ]

        subitems = []
        for cmd, label in commands:
            item = Nautilus.MenuItem(
                name=f"TildrExtension::{cmd}",
                label=label,
                tip=f"tildr {cmd}",
            )

            def on_activate(*_, c=cmd, r=rels):
                subprocess.Popen(
                    [TILDR_PATH, c] + r,
                    close_fds=True,
                    start_new_session=True,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    stdin=subprocess.DEVNULL,
                )

            item.connect("activate", on_activate)
            subitems.append(item)
            submenu.append_item(item)

        self._items = [root] + subitems
        return [root]

    def get_background_items(self, folder):
        return []
