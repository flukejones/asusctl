declare const global: any, imports: any;
declare var asusctlGexInstance: any;
//@ts-ignore
const Me = imports.misc.extensionUtils.getCurrentExtension();

import * as Resources from './resources';

const {Gio, GLib} = imports.gi;

export class ChargingLimit {
    asusLinuxProxy: any = null; // type: Gio.DbusProxy (donno how to add)
    connected: boolean = false;
    lastState: number = 100;
    pollerDelayTicks: number = 0;
    timeoutChargePoller: number | null = null;

    constructor() {
        // nothing for now
    }

    public getChargingLimit() {
        if (this.isRunning()) {
            try {
                let currentState = this.asusLinuxProxy.LimitSync().toString().trim();

                return currentState;
            } catch (e) {
                //@ts-ignore
                log(`Failed to fetch Charging Limit!`, e);
            }
        }

        return this.lastState;
    }

    public setChargingLimit(limit: number) {
        if (this.isRunning()) {
            try {
                if (limit > 0 && this.lastState !== limit) {
                    // update state
                    this.lastState = limit;
                }
                return this.asusLinuxProxy.SetLimitSync(limit);
            } catch (e) {
                //@ts-ignore
                log(`Profile DBus set power profile failed!`, e);
            }
        }
    }

    updateChargingLimit(curState: number) {
        // return false;
        if (curState > 0 && this.lastState !== curState) {
            // disable the signal handler so we don't run in an infinite loop
            // of notifying, setting, notifying, setting...
            asusctlGexInstance.chargingLimit.chargingLimitSlider.block_signal_handler(asusctlGexInstance.chargingLimit._sliderChangedId);
            asusctlGexInstance.chargingLimit.chargingLimitSlider.value = curState/100;
            asusctlGexInstance.chargingLimit.chargingLimitSlider.unblock_signal_handler(asusctlGexInstance.chargingLimit._sliderChangedId);

            asusctlGexInstance.chargingLimit.chargeLimitLabel.set_text(`${curState}%`);

            // update state
            this.lastState = curState;
        }
    }

    pollerChargingLimit() {
        if(this.isRunning() && this.pollerDelayTicks <= 0){
            try {
                let currentLimit = this.getChargingLimit();
                if (currentLimit !== this.lastState){
                    this.updateChargingLimit(currentLimit);

                    // Panel.Actions.notify(
                    //     'ASUS Notebook Control',
                    //     `Charging Limit changed to ${currentLimit}%`,
                    //     'scalable/battery-symbolic.svg'
                    // );
                }
            } catch (e) {
                //@ts-ignore
                log(`Charging Limit poller init failed!`, e);
            } finally {
                return this.isRunning() ? GLib.SOURCE_CONTINUE : GLib.SOURCE_REMOVE;
            }
        } else if (this.isRunning() && this.pollerDelayTicks > 0) {
            this.pollerDelayTicks--;
            return GLib.SOURCE_CONTINUE;
        } else {
            return GLib.SOURCE_REMOVE;
        }
    }

    isRunning(): boolean {
        return this.connected;
    }

    async start() {
        //@ts-ignore
        log(`Starting Charging Limit DBus client...`);

        try {
            // creating the proxy
            let xml = Resources.File.DBus('org-asuslinux-charge-4')
            this.asusLinuxProxy = new Gio.DBusProxy.makeProxyWrapper(xml)(
                Gio.DBus.system,
                'org.asuslinux.Daemon',
                '/org/asuslinux/Charge'
            );

            this.connected = true;
            this.lastState = this.getChargingLimit();

            this.asusLinuxProxy.connectSignal(
                "NotifyCharge",
                (proxy: any = null, name: string, data: string) => {
                    if (proxy) {
                        //@ts-ignore
                        log(`Charging Limit has changed to ${data}% (${name}).`);
                        this.updateChargingLimit(parseInt(data));
                    }
                }
            );

            try {
                this.timeoutChargePoller = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 5, this.pollerChargingLimit.bind(this));
            } catch (e) {
                //@ts-ignore
                log(`Charging Limit DBus Poller initialization failed!`, e);
            }
        } catch (e) {
            //@ts-ignore
            log(`Charging Limit DBus initialization failed!`, e);
        }
    }

    stop() {
        //@ts-ignore
        log(`Stopping Charging Limit DBus client...`);

        if (this.isRunning()) {
            this.connected = false;
            this.asusLinuxProxy = null;
            this.lastState = 100;
            GLib.Source.remove(this.timeoutChargePoller);
            this.timeoutChargePoller = null;
        }
    }
}