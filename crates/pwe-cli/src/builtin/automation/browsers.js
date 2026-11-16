function value(object, key) {
    const result = object[key]();
    if (result === null || result === undefined) return '';
    if (result instanceof Date) return result.toISOString();
    return String(result);
}

function each(collection, callback) {
    const results = [];
    for (let index = 0, count = collection.length; index < count; index++) results.push(callback(collection[index]));
    return results;
}

function tabs(browser) {
    const app = Application(browser === 'chrome' ? 'Google Chrome' : 'Arc');
    if (!app.running()) throw new Error(browser + ' is not running');
    const rows = [], seen = {};
    function append(window, collection, space) {
        const windowId = value(window, 'id');
        const ids = collection.id(), titles = collection.title(), urls = collection.url();
        const locations = browser === 'arc' ? collection.location() : [];
        for (let index = 0; index < ids.length; index++) {
            const key = windowId + ':' + String(ids[index]);
            if (seen[key]) continue;
            seen[key] = true;
            rows.push({id: key, window: windowId, tab: String(ids[index]),
                title: titles[index], url: urls[index], space: space, location: locations[index] || ''});
        }
    }
    each(app.windows, function (window) {
        if (browser === 'arc') each(window.spaces, function (space) {
            append(window, space.tabs, {id:value(space, 'id'), title:value(space, 'title')});
        });
        append(window, window.tabs, null);
    });
    return rows;
}

// Arc fails "AppleEvent handler failed" when a window's index is set; its active window is
// already in front once the app is activated, so only Chrome needs the raise.
function raise(window) {
    try { window.index = 1; } catch (error) { /* Arc */ }
}

function focus(browser, item) {
    const app = Application(browser === 'chrome' ? 'Google Chrome' : 'Arc');
    if (!app.running()) throw new Error(browser + ' is not running');
    const windows = app.windows;
    for (let w = 0; w < windows.length; w++) {
        const window = windows[w];
        if (value(window, 'id') !== item.window) continue;
        let collection = window.tabs;
        if (browser === 'arc' && item.space) {
            const spaces = window.spaces;
            for (let s = 0; s < spaces.length; s++) {
                if (value(spaces[s], 'id') === item.space.id) {
                    collection = spaces[s].tabs;
                    for (let t = 0; t < collection.length; t++) if (value(collection[t], 'id') === item.tab) {
                        spaces[s].focus(); collection[t].select(); raise(window); app.activate(); return true;
                    }
                }
            }
        }
        for (let t = 0; t < collection.length; t++) if (value(collection[t], 'id') === item.tab) {
            if (browser === 'arc') collection[t].select();
            else window.activeTabIndex = t + 1;
            raise(window); app.activate(); return true;
        }
    }
    throw new Error('Tab closed or moved. Open the tabs picker again.');
}

function run(argv) {
    const request = JSON.parse(argv[0]);
    if (request.action === 'contacts') return JSON.stringify(contacts());
    if (request.action === 'tabs') return JSON.stringify(tabs(request.browser));
    if (request.action === 'focus') return JSON.stringify(focus(request.browser, request.item));
    throw new Error('Unknown macOS action');
}
