import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk';
import {ExtensionPreferences, gettext as _} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

const BUS_NAME = 'io.github.0x5c0f.KeyStats';
const OBJ_PATH = '/io/github/0x5c0f/KeyStats';
const IFACE = 'io.github.0x5c0f.KeyStats1';

function dbusCall(method) {
    let bus = Gio.DBus.session;
    return bus.call_sync(
        BUS_NAME, OBJ_PATH, IFACE, method,
        null, new GLib.VariantType('(b)'),
        Gio.DBusCallFlags.NONE, -1, null
    );
}

function dbusCallChecked(method) {
    try {
        dbusCall(method);
    } catch (e) {
        // Show error to user
        let dlg = new Adw.MessageDialog({
            heading: _('Error'),
            body: _('Could not reach the KeyStats daemon.\nIs it running? Try: keystatsctl status'),
            close_response: 'ok',
            modal: true,
        });
        dlg.add_response('ok', _('OK'));
        dlg.present();
    }
}

export default class KeyStatsPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        window._settings = this.getSettings();
        const s = window._settings;

        const page = new Adw.PreferencesPage({
            title: _('KeyStats'),
            icon_name: 'input-keyboard-symbolic',
        });
        window.add(page);

        /* ── Panel Display ──────────────────────────── */

        const displayGroup = new Adw.PreferencesGroup({
            title: _('Panel Display'),
            description: _('Choose what appears in the top bar.'),
        });
        page.add(displayGroup);

        const showKeysRow = new Adw.SwitchRow({
            title: _('Show Key Presses'),
            subtitle: _('Display key press count in the top bar panel'),
        });
        displayGroup.add(showKeysRow);
        s.bind('show-keys', showKeysRow, 'active', Gio.SettingsBindFlags.DEFAULT);

        const showClicksRow = new Adw.SwitchRow({
            title: _('Show Click Count'),
            subtitle: _('Display mouse click count in the top bar panel'),
        });
        displayGroup.add(showClicksRow);
        s.bind('show-clicks', showClicksRow, 'active', Gio.SettingsBindFlags.DEFAULT);

        /* ── Refresh ────────────────────────────────── */

        const refreshGroup = new Adw.PreferencesGroup({
            title: _('Refresh'),
            description: _('How often to poll the daemon for new data.'),
        });
        page.add(refreshGroup);

        const refreshRow = new Adw.SpinRow({
            title: _('Refresh Interval'),
            subtitle: _('Milliseconds between data updates'),
            adjustment: new Gtk.Adjustment({
                lower: 500, upper: 5000, step_increment: 100,
            }),
            value: s.get_int('refresh-interval'),
        });
        refreshGroup.add(refreshRow);
        refreshRow.connect('changed', (spin) => {
            s.set_int('refresh-interval', spin.get_value());
        });

        /* ── Appearance ─────────────────────────────── */

        const appearGroup = new Adw.PreferencesGroup({
            title: _('Appearance'),
        });
        page.add(appearGroup);

        // Dynamic color
        // TODO: implement in a future release — should read the GSettings
        // value and apply the system accent color to chart highlights, KPS
        // badge, and hero value text in the popup via St dynamic theming.
        const colorRow = new Adw.SwitchRow({
            title: _('Dynamic Accent Color'),
            subtitle: _('Use system accent color for highlights'),
        });
        appearGroup.add(colorRow);
        s.bind('dynamic-color', colorRow, 'active', Gio.SettingsBindFlags.DEFAULT);

        /* ── Data Management ────────────────────────── */

        const dataGroup = new Adw.PreferencesGroup({
            title: _('Data Management'),
            description: _('Reset or clear your collected statistics.'),
        });
        page.add(dataGroup);

        // Reset Today
        const resetRow = new Adw.ActionRow({
            title: _('Reset Today'),
            subtitle: _('Clear only today\'s counts — history is preserved'),
        });
        const resetBtn = new Gtk.Button({
            label: _('Reset'),
            valign: Gtk.Align.CENTER,
            css_classes: ['destructive-action'],
        });
        resetBtn.connect('clicked', () => {
            dbusCallChecked('ResetToday');
        });
        resetRow.add_suffix(resetBtn);
        dataGroup.add(resetRow);

        // Clear All Data
        const clearRow = new Adw.ActionRow({
            title: _('Clear All Data'),
            subtitle: _('Permanently delete today and all history'),
        });
        const clearBtn = new Gtk.Button({
            label: _('Clear All'),
            valign: Gtk.Align.CENTER,
            css_classes: ['destructive-action'],
        });
        clearBtn.connect('clicked', () => {
            let dlg = new Adw.MessageDialog({
                transient_for: window,
                heading: _('Clear All Data?'),
                body: _('This will permanently delete today\'s statistics and all history. This action cannot be undone.'),
                close_response: 'cancel',
                modal: true,
            });
            dlg.add_response('cancel', _('Cancel'));
            dlg.add_response('clear', _('Clear All'));
            dlg.set_response_appearance('clear', Adw.ResponseAppearance.DESTRUCTIVE);
            dlg.connect('response', (_, resp) => {
                if (resp === 'clear') {
                    dbusCallChecked('ClearAllData');
                }
            });
            dlg.present();
        });
        clearRow.add_suffix(clearBtn);
        dataGroup.add(clearRow);
    }
}
