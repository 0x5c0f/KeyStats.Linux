import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import Clutter from 'gi://Clutter';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const BUS_NAME = 'io.github.x0x5c0f.KeyStats';
const OBJ_PATH = '/io/github/0x5c0f/KeyStats';
const IFACE = 'io.github.x0x5c0f.KeyStats1';

/* ── Theme detection ────────────────────────────────── */

function isSystemDark() {
    try {
        let s = new Gio.Settings({schema_id: 'org.gnome.desktop.interface'});
        let scheme = s.get_string('color-scheme');
        return scheme === 'prefer-dark';
    } catch (_) { return true; }
}

// Listen to theme changes
function onThemeChanged(ext) {
    let dark = isSystemDark();
    let box = ext._popupBox;
    if (!box) return;
    if (dark) {
        box.remove_style_class_name('ks-light');
        box.add_style_class_name('ks-dark');
    } else {
        box.remove_style_class_name('ks-dark');
        box.add_style_class_name('ks-light');
    }
}

/* ── Formatters ─────────────────────────────────────── */

function fmtNum(n) {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return String(n);
}
// Mouse distance: px → m → km (matches macOS baseMetersPerPixel = 0.000264583)
const M_PER_PX = 0.000264583;
function fmtMouseDist(px) {
    let m = px * M_PER_PX;
    if (m >= 1000) return (m / 1000).toFixed(2) + ' km';
    if (px >= 1000) return m.toFixed(1) + ' m';
    return Math.round(px) + ' px';
}
// Scroll distance: px → kPx (matches macOS)
function fmtScrollDist(px) {
    if (px >= 10000) return (px / 1000).toFixed(1) + ' kPx';
    return Math.round(px) + ' px';
}

/* ── Key name abbreviation ────────────────────────────── */

const KEY_ABBREV = {
    'Backspace': 'Bksp',
    'LeftShift': 'LShift',
    'RightShift': 'RShift',
    'LeftCtrl': 'LCtrl',
    'RightCtrl': 'RCtrl',
    'LeftAlt': 'LAlt',
    'RightAlt': 'RAlt',
    'CapsLock': 'Caps',
    'PageUp': 'PgUp',
    'PageDown': 'PgDn',
    'NumLock': 'Num',
    'ScrollLock': 'ScrLk',
    'PrintScreen': 'PrtSc',
    'Escape': 'Esc',
    'Delete': 'Del',
    'Insert': 'Ins',
    'Multiply': '*',
    'Divide': '/',
    'Subtract': '-',
    'Decimal': '.',
};

function abbrevKey(name) {
    return KEY_ABBREV[name] ?? name;
}

/* ── Widget builders ────────────────────────────────── */

function sectionLabel(text) {
    return new St.Label({text, style_class: 'ks-section'});
}

function heroCard(title, value, fmtFn) {
    let card = new St.BoxLayout({vertical: true, style_class: 'ks-hero'});
    let lbl = new St.Label({text: title, style_class: 'ks-hero-title'});
    let val = new St.Label({
        text: fmtFn ? fmtFn(value ?? 0) : String(value ?? 0),
        style_class: 'ks-hero-value',
    });
    card.add_child(lbl);
    card.add_child(val);
    return card;
}

function clickTile(label, value) {
    let tile = new St.BoxLayout({vertical: true, style_class: 'ks-click-tile', x_expand: true});
    let lbl = new St.Label({text: label, style_class: 'ks-click-label'});
    let val = new St.Label({text: String(value ?? 0), style_class: 'ks-click-value'});
    tile.add_child(lbl);
    tile.add_child(val);
    return tile;
}

function distCard(title, value, fmtFn) {
    let card = new St.BoxLayout({vertical: true, style_class: 'ks-dist'});
    let lbl = new St.Label({text: title, style_class: 'ks-dist-title'});
    let val = new St.Label({
        text: fmtFn ? fmtFn(value ?? 0) : String(value ?? 0),
        style_class: 'ks-dist-value',
    });
    card.add_child(lbl);
    card.add_child(val);
    return card;
}

function actionBtn(label) {
    return new St.Button({label, style_class: 'ks-btn'});
}

/* ── Extension ──────────────────────────────────────── */

export default class KeyStatsExtension extends Extension {
    enable() {
        this._settings = this.getSettings();
        this._themeSignal = null;

        // Panel button
        this._button = new PanelMenu.Button(0.0, 'KeyStats', false);
        let pb = new St.BoxLayout({style_class: 'ks-panel-box'});
        this._keyLabel = new St.Label({text: '…', style_class: 'ks-panel-label', y_align: Clutter.ActorAlign.CENTER});
        this._clickLabel = new St.Label({text: '', style_class: 'ks-panel-label', y_align: Clutter.ActorAlign.CENTER});
        pb.add_child(this._keyLabel);
        pb.add_child(this._clickLabel);
        this._button.add_child(pb);
        Main.panel.addToStatusArea('keystats', this._button, 0, 'right');

        this._buildPopup();

        let interval = this._settings.get_int('refresh-interval');
        this._timeoutId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, interval, () => this._poll());
        this._poll();

        // Theme
        onThemeChanged(this);
        try {
            let sys = new Gio.Settings({schema_id: 'org.gnome.desktop.interface'});
            this._themeSignal = sys.connect('changed::color-scheme', () => onThemeChanged(this));
        } catch (_) {}

        console.log('[KeyStats] enabled');
    }

    _buildPopup() {
        let box = this._button.menu.box;
        this._popupBox = box;
        box.add_style_class_name('ks-popup');

        // 1. Header: title + connection status + KPS/CPS
        this._header = new St.BoxLayout({x_expand: true});
        let title = new St.Label({text: 'KeyStats', style: 'font-size: 14px; font-weight: 700;'});
        // Connection dot
        this._connDot = new St.BoxLayout({style_class: 'ks-conn-dot'});
        this._kpsBadge = new St.BoxLayout({style_class: 'ks-kps'});
        this._kpsLbl = new St.Label({text: '--', style_class: 'ks-kps-text'});
        this._kpsBadge.add_child(this._kpsLbl);
        this._header.add_child(title);
        this._header.add_child(this._connDot);
        this._header.add_child(this._kpsBadge);
        box.add_child(this._header);

        // 2. Hero
        this._hero = new St.BoxLayout({style_class: 'ks-hero-row'});
        box.add_child(this._hero);

        // 3. Click detail
        box.add_child(sectionLabel(this.gettext('Mouse Click Detail')));
        this._clickRow = new St.BoxLayout({style_class: 'ks-click-row'});
        box.add_child(this._clickRow);
        this._sideRow = new St.BoxLayout({style_class: 'ks-click-row'});
        box.add_child(this._sideRow);

        // 4. Distance
        box.add_child(sectionLabel(this.gettext('Distance')));
        this._distRow = new St.BoxLayout({style_class: 'ks-dist-row'});
        box.add_child(this._distRow);

        // 5. Key Breakdown — placeholder for future feature
        box.add_child(sectionLabel(this.gettext('Key Breakdown')));
        this._keySection = new St.BoxLayout({vertical: true, style_class: 'ks-key-grid'});
        box.add_child(this._keySection);

        // 6. History
        box.add_child(sectionLabel(this.gettext('History (7 days)')));
        this._histBox = new St.BoxLayout({vertical: true});
        box.add_child(this._histBox);

        // 7. Actions
        let sep = new St.BoxLayout({style_class: 'ks-sep'});
        box.add_child(sep);
        this._actions = new St.BoxLayout({style_class: 'ks-actions'});
        let prefs = actionBtn(this.gettext('Preferences'));
        prefs.connect('clicked', () => {
            try {
                Gio.DBus.session.call('org.gnome.Shell.Extensions', '/org/gnome/Shell/Extensions',
                    'org.gnome.Shell.Extensions', 'LaunchExtensionPrefs',
                    new GLib.Variant('(s)', [this.metadata.uuid]), null,
                    Gio.DBusCallFlags.NONE, -1, null, null);
            } catch (_) {}
        });
        this._actions.add_child(prefs);
        box.add_child(this._actions);
    }

    _poll() {
        try {
            let [t] = Gio.DBus.session.call_sync(
                BUS_NAME, OBJ_PATH, IFACE, 'GetTodayStats',
                null, new GLib.VariantType('(a{sv})'), Gio.DBusCallFlags.NONE, -1, null
            ).deepUnpack();

            let keys = t.keyPresses?.deepUnpack() ?? 0;
            let clicks = t.totalClicks?.deepUnpack() ?? 0;
            let lc = t.leftClicks?.deepUnpack() ?? 0;
            let mc = t.middleClicks?.deepUnpack() ?? 0;
            let rc = t.rightClicks?.deepUnpack() ?? 0;
            let sbc = t.sideBackClicks?.deepUnpack() ?? 0;
            let sfc = t.sideForwardClicks?.deepUnpack() ?? 0;
            let md = t.mouseDistance?.deepUnpack() ?? 0;
            let sd = t.scrollDistance?.deepUnpack() ?? 0;
            let kps = t.currentKPS?.deepUnpack() ?? 0;
            let cps = t.currentCPS?.deepUnpack() ?? 0;

            let sk = this._settings.get_boolean('show-keys');
            let sc = this._settings.get_boolean('show-clicks');
            this._keyLabel.text = sk ? this.gettext('K') + fmtNum(keys) : '';
            this._clickLabel.text = sc ? ' ' + this.gettext('C') + fmtNum(clicks) : '';
            this._kpsLbl.text = this.gettext('K') + kps + ' ' + this.gettext('C') + cps + this.gettext('/s');
            // Connection: green when daemon reachable
            this._connDot.remove_style_class_name('ks-conn-err');
            this._connDot.add_style_class_name('ks-conn-ok');

            this._hero.destroy_all_children();
            this._hero.add_child(heroCard(this.gettext('Key Presses'), keys, fmtNum));
            this._hero.add_child(heroCard(this.gettext('Mouse Clicks'), clicks, fmtNum));

            this._clickRow.destroy_all_children();
            this._clickRow.add_child(clickTile(this.gettext('Left'), lc));
            this._clickRow.add_child(clickTile(this.gettext('Middle'), mc));
            this._clickRow.add_child(clickTile(this.gettext('Right'), rc));
            this._sideRow.destroy_all_children();
            if (sbc > 0 || sfc > 0) {
                this._sideRow.add_child(clickTile(this.gettext('Side Back'), sbc));
                this._sideRow.add_child(clickTile(this.gettext('Side Fwd'), sfc));
            }

            this._distRow.destroy_all_children();
            this._distRow.add_child(distCard(this.gettext('Mouse Dist'), md, fmtMouseDist));
            this._distRow.add_child(distCard(this.gettext('Scroll Dist'), sd, fmtScrollDist));

            this._fetchHistory();
            this._fetchKeyBreakdown();
        } catch (e) {
            this._keyLabel.text = this.gettext('K') + '--';
            this._clickLabel.text = this.gettext('C') + '--';
            this._kpsLbl.text = this.gettext('offline');
            this._connDot.remove_style_class_name('ks-conn-ok');
            this._connDot.add_style_class_name('ks-conn-err');
        }
        return true;
    }

    _fetchKeyBreakdown() {
        try {
            let [json] = Gio.DBus.session.call_sync(
                BUS_NAME, OBJ_PATH, IFACE, 'GetTopKeys',
                new GLib.Variant('(u)', [15]),
                new GLib.VariantType('(s)'),
                Gio.DBusCallFlags.NONE, -1, null
            ).deepUnpack();
            let keys = JSON.parse(json ?? '[]');
            this._keySection.destroy_all_children();

            if (!keys || keys.length === 0) {
                this._keySection.add_child(
                    new St.Label({text: this.gettext('No keys recorded yet'), style_class: 'ks-dim'})
                );
                return;
            }

            // 3 columns of 5 keys — matching macOS/Windows layout
            let colCount = 3;
            let maxPerCol = 5;
            let cols = [];
            for (let i = 0; i < colCount; i++) {
                cols.push(new St.BoxLayout({vertical: true, style_class: 'ks-key-col'}));
            }

            for (let i = 0; i < Math.min(keys.length, colCount * maxPerCol); i++) {
                // Row-major fill: keys flow left→right, top→bottom
                let colIdx = i % colCount;
                let k = keys[i];
                let row = new St.BoxLayout({style_class: 'ks-key-row-item'});
                let badge = new St.Label({
                    text: abbrevKey(k.key_name ?? '?'),
                    style_class: 'ks-key-badge',
                });
                let cnt = new St.Label({
                    text: fmtNum(k.count ?? 0),
                    style_class: 'ks-key-badge-count',
                });
                row.add_child(badge);
                row.add_child(cnt);
                cols[colIdx].add_child(row);
            }

            // Add separators between columns
            let grid = new St.BoxLayout({style_class: 'ks-key-grid-row'});
            for (let i = 0; i < colCount; i++) {
                if (i > 0) {
                    grid.add_child(new St.BoxLayout({style_class: 'ks-key-col-sep'}));
                }
                grid.add_child(cols[i]);
            }
            this._keySection.add_child(grid);
        } catch (_) {
            // non-critical, stay with current display
        }
    }

    _fetchHistory() {
        try {
            let [json] = Gio.DBus.session.call_sync(
                BUS_NAME, OBJ_PATH, IFACE, 'GetHistory',
                new GLib.Variant('(u)', [7]), new GLib.VariantType('(s)'),
                Gio.DBusCallFlags.NONE, -1, null
            ).deepUnpack();
            let data = JSON.parse(json ?? '[]');
            this._histBox.destroy_all_children();
            if (!data || data.length === 0) {
                this._histBox.add_child(new St.Label({text: this.gettext('No history yet'), style_class: 'ks-dim'}));
                return;
            }
            let maxK = Math.max(1, ...data.map(d => d.key_presses ?? 0));
            let wrap = new St.BoxLayout({style_class: 'ks-hist-wrap', vertical: true});
            let bars = new St.BoxLayout({style_class: 'ks-hist-bars'});
            for (let d of data.slice().reverse()) {
                let col = new St.BoxLayout({vertical: true, style: 'margin-right: 5px;'});
                let dt = new St.Label({text: (d.date ?? '').slice(5), style_class: 'ks-hist-date'});
                let h = Math.max(3, ((d.key_presses ?? 0) / maxK * 36));
                let bar = new St.BoxLayout({
                    style: 'background-color: #0078d4; border-radius: 2px;'
                        + ' min-height: ' + h + 'px; min-width: 24px;',
                });
                let lbl = new St.Label({text: fmtNum(d.key_presses ?? 0), style_class: 'ks-hist-label'});
                col.add_child(dt);
                col.add_child(bar);
                col.add_child(lbl);
                bars.add_child(col);
            }
            wrap.add_child(bars);
            this._histBox.add_child(wrap);
        } catch (_) {}
    }

    disable() {
        if (this._themeSignal) {
            try {
                let s = new Gio.Settings({schema_id: 'org.gnome.desktop.interface'});
                s.disconnect(this._themeSignal);
            } catch (_) {}
            this._themeSignal = null;
        }
        if (this._timeoutId) { GLib.Source.remove(this._timeoutId); this._timeoutId = null; }
        this._keyLabel?.destroy(); this._keyLabel = null;
        this._clickLabel?.destroy(); this._clickLabel = null;
        this._header?.destroy(); this._header = null;
        this._connDot?.destroy(); this._connDot = null;
        this._kpsLbl?.destroy(); this._kpsLbl = null;
        this._kpsBadge?.destroy(); this._kpsBadge = null;
        this._hero?.destroy(); this._hero = null;
        this._clickRow?.destroy(); this._clickRow = null;
        this._sideRow?.destroy(); this._sideRow = null;
        this._distRow?.destroy(); this._distRow = null;
        this._keySection?.destroy(); this._keySection = null;
        this._histBox?.destroy(); this._histBox = null;
        this._actions?.destroy(); this._actions = null;
        this._popupBox = null;
        if (this._button) { this._button.destroy(); this._button = null; }
        this._settings = null;
        console.log('[KeyStats] disabled');
    }
}
