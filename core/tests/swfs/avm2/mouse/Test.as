package {
    public class Test {
    }
}

import flash.ui.Mouse;
import flash.ui.MouseCursor;
import flash.ui.MouseCursorData;

trace("// Mouse.supportsCursor")
trace(Mouse.supportsCursor);
trace("// Mouse.supportsNativeCursor")
trace(Mouse.supportsNativeCursor);

trace("// Mouse.cursor")
trace(Mouse.cursor);

Mouse.cursor = MouseCursor.IBEAM;
trace("// Mouse.cursor (after set to MouseCursor.IBEAM)")
trace(Mouse.cursor);
