import GObject from 'gi://GObject';
import St from 'gi://St';
import Gio from 'gi://Gio';
import {Extension, gettext as _} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

const SovereignIndicator = GObject.registerClass(
class SovereignIndicator extends PanelMenu.Button {
    _init() {
        super._init(0.0, _('Sovereign Control'));

        // Icon in the panel
        let icon = new St.Icon({
            icon_name: 'weather-clear-symbolic',
            style_class: 'system-status-icon',
        });
        this.add_child(icon);

        // D-Bus proxies
        this._boreasProxy = null;
        this._prometheusProxy = null;
        this._connectDBus();

        // Thermal Control Section (Boreas)
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        let thermalLabel = new PopupMenu.PopupMenuItem(_('⚡ THERMAL CONTROL'), {
            reactive: false,
            can_focus: false
        });
        this.menu.addMenuItem(thermalLabel);

        this._fanSilentItem = new PopupMenu.PopupMenuItem(_('Silent Fans'));
        this._fanBalancedItem = new PopupMenu.PopupMenuItem(_('Balanced Fans'));
        this._fanMaxItem = new PopupMenu.PopupMenuItem(_('MAX POWER Fans'));
        this._fanAutoItem = new PopupMenu.PopupMenuItem(_('Auto Fans'));

        this._fanSilentItem.connect('activate', () => this._setFanProfile('silent'));
        this._fanBalancedItem.connect('activate', () => this._setFanProfile('balanced'));
        this._fanMaxItem.connect('activate', () => this._setFanProfile('maxpower'));
        this._fanAutoItem.connect('activate', () => this._setFanProfile('auto'));

        this.menu.addMenuItem(this._fanSilentItem);
        this.menu.addMenuItem(this._fanBalancedItem);
        this.menu.addMenuItem(this._fanMaxItem);
        this.menu.addMenuItem(this._fanAutoItem);

        // Performance Control Section (Prometheus)
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        let perfLabel = new PopupMenu.PopupMenuItem(_('🔥 PERFORMANCE CONTROL'), {
            reactive: false,
            can_focus: false
        });
        this.menu.addMenuItem(perfLabel);

        this._cpuSilentItem = new PopupMenu.PopupMenuItem(_('Silent CPU'));
        this._cpuBalancedItem = new PopupMenu.PopupMenuItem(_('Balanced CPU'));
        this._cpuWarSpeedItem = new PopupMenu.PopupMenuItem(_('WARSPEED CPU'));

        this._cpuSilentItem.connect('activate', () => this._setCpuProfile('silent'));
        this._cpuBalancedItem.connect('activate', () => this._setCpuProfile('balanced'));
        this._cpuWarSpeedItem.connect('activate', () => this._setCpuProfile('warspeed'));

        this.menu.addMenuItem(this._cpuSilentItem);
        this.menu.addMenuItem(this._cpuBalancedItem);
        this.menu.addMenuItem(this._cpuWarSpeedItem);

        // Combined Profiles
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        let combinedLabel = new PopupMenu.PopupMenuItem(_('⚔️ COMBINED SOVEREIGNTY'), {
            reactive: false,
            can_focus: false
        });
        this.menu.addMenuItem(combinedLabel);

        this._totalWarItem = new PopupMenu.PopupMenuItem(_('TOTAL WAR'));
        this._totalWarItem.connect('activate', () => this._setTotalWar());
        this.menu.addMenuItem(this._totalWarItem);
    }

    _connectDBus() {
        // Boreas proxy
        const BoreasProxy = Gio.DBusProxy.makeProxyWrapper(
            `<node>
                <interface name="org.jesternet.Boreas">
                    <method name="SetFanProfile">
                        <arg type="s" direction="in" name="profile"/>
                        <arg type="s" direction="out" name="result"/>
                    </method>
                </interface>
            </node>`
        );

        // Prometheus proxy
        const PrometheusProxy = Gio.DBusProxy.makeProxyWrapper(
            `<node>
                <interface name="org.jesternet.Prometheus">
                    <method name="SetPerformanceProfile">
                        <arg type="s" direction="in" name="profile"/>
                        <arg type="s" direction="out" name="result"/>
                    </method>
                </interface>
            </node>`
        );

        try {
            this._boreasProxy = new BoreasProxy(
                Gio.DBus.system,
                'org.jesternet.Boreas',
                '/org/jesternet/Boreas'
            );
        } catch (e) {
            logError(e, 'Failed to connect to Boreas daemon');
        }

        try {
            this._prometheusProxy = new PrometheusProxy(
                Gio.DBus.system,
                'org.jesternet.Prometheus',
                '/org/jesternet/Prometheus'
            );
        } catch (e) {
            logError(e, 'Failed to connect to Prometheus daemon');
        }
    }

    _setFanProfile(profile) {
        if (!this._boreasProxy) {
            Main.notify('Sovereign Control', 'Boreas daemon not connected');
            return;
        }

        this._boreasProxy.SetFanProfileRemote(profile, (result, error) => {
            if (error) {
                Main.notify('Boreas', `Error: ${error.message}`);
            } else {
                Main.notify('Boreas', result[0]);
            }
        });
    }

    _setCpuProfile(profile) {
        if (!this._prometheusProxy) {
            Main.notify('Sovereign Control', 'Prometheus daemon not connected');
            return;
        }

        this._prometheusProxy.SetPerformanceProfileRemote(profile, (result, error) => {
            if (error) {
                Main.notify('Prometheus', `Error: ${error.message}`);
            } else {
                Main.notify('Prometheus', result[0]);
            }
        });
    }

    _setTotalWar() {
        Main.notify('TOTAL WAR', 'Engaging maximum sovereignty...');

        // Set fans to MAX POWER
        if (this._boreasProxy) {
            this._boreasProxy.SetFanProfileRemote('maxpower', (result, error) => {
                if (!error) {
                    log('Boreas: MAX POWER engaged');
                }
            });
        }

        // Set CPU to WARSPEED
        if (this._prometheusProxy) {
            this._prometheusProxy.SetPerformanceProfileRemote('warspeed', (result, error) => {
                if (!error) {
                    log('Prometheus: WARSPEED engaged');
                }
            });
        }

        // Notify after 500ms
        setTimeout(() => {
            Main.notify('TOTAL WAR', 'Full sovereign performance active!');
        }, 500);
    }

    destroy() {
        super.destroy();
    }
});

export default class SovereignExtension extends Extension {
    enable() {
        this._indicator = new SovereignIndicator();
        Main.panel.addToStatusArea(this.uuid, this._indicator);
    }

    disable() {
        this._indicator?.destroy();
        this._indicator = null;
    }
}
