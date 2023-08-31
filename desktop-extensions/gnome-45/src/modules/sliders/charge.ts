import { Power } from "../dbus/power";
import { addQuickSettingsItems } from "../helpers";
import GObject from 'gi://GObject';
import * as QuickSettings from 'resource:///org/gnome/shell/ui/quickSettings.js';
import * as AsusExtension from "../../extension.js";

export const SliderChargeLevel = GObject.registerClass(
    class SliderChargeLevel extends QuickSettings.QuickSlider {
        private _dbus_power: Power;

        constructor(dbus_power: Power) {
            super({
                iconName: "selection-mode-symbolic",
            });
            this._dbus_power = dbus_power;

            this._sliderChangedId = this.slider.connect("drag-end",
                this._onSliderChanged.bind(this));

            // Binding the slider to a GSettings key

            AsusExtension.extension._settings.connect("changed::charge-level",
                this._onSettingsChanged.bind(this));

            // Set an accessible name for the slider
            this.slider.accessible_name = "Charge level";

            this._sync();
            this._onSettingsChanged();

            addQuickSettingsItems([this], 2);
        }

        _onSettingsChanged() {
            // Prevent the slider from emitting a change signal while being updated
            this.slider.block_signal_handler(this._sliderChangedId);
            this.slider.value =  AsusExtension.extension._settings.get_uint("charge-level") / 100.0;
            this.slider.unblock_signal_handler(this._sliderChangedId);
        }

        _onSliderChanged() {
            // Assuming our GSettings holds values between 0..100, adjust for the
            // slider taking values between 0..1
            const percent = Math.floor(this.slider.value * 100);
            const stored = Math.floor( AsusExtension.extension._settings.get_uint("charge-level") / 100.0);
            if (this.slider.value !== stored)
                this._dbus_power.setChargingLimit(percent);
                AsusExtension.extension._settings.set_uint("charge-level", percent);
        }

        _sync() {
            const value = this._dbus_power.getChargingLimit();
            if (this.slider.value !== value / 100)
            AsusExtension.extension._settings.set_uint("charge-level", value);
        }
    });