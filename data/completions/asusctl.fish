# Author: AlenPaulVarghese <alenpaul2001@gmail.com>

set -l progname asusctl

set -l noopt "not __fish_contains_opt -s -s h -s v -s s -s k -s f -s c help version show-supported kbd-bright fan-mode chg-limit; and not __fish_seen_subcommand_from led-mode profile graphics;"


set -l gmod_options '__fish_contains_opt -s m mode;'
set -l fan_options '__fish_contains_opt -s f fan-mode;'
set -l led_options '__fish_seen_subcommand_from led-mode;'
set -l profile_options '__fish_seen_subcommand_from profile;'
set -l keyboard_options '__fish_contains_opt -s k kbd-bright;'
set -l graphics_options '__fish_seen_subcommand_from graphics;'

set -l fan_modes 'silent normal boost'
set -l brightness_modes 'off low med high'
set -l led_modes 'static breathe strobe rainbow comet'
set -l graphics_modes 'nvidia hybird compute integrated'


complete -c $progname -e
complete -c $progname -f

# asusctl completion
complete -c $progname -s h -f -l help -n "$noopt" -d "print help message"
complete -c $progname -s v -f -l version -n "$noopt" -d "show program version number"
complete -c $progname -s s -f -l show-supported -n "$noopt" -d "show supported functions of this laptop"
complete -c $progname -s k -f -l kbd-bright -n "$noopt" -d "set led brightness"
complete -c $progname -s f -f -l fan-mode -n "$noopt" -d "set fan mode independent of profile"
complete -c $progname -s c -f -l chg-limit -n "$noopt" -d "set charge limit <20-100>"
complete -c $progname -f -a "led-mode" -n "$noopt" -d "Set the keyboard lighting from built-in modes"
complete -c $progname -f -a "profile" -n "$noopt" -d "Create and configure profiles"
complete -c $progname -f -a "graphics" -n "$noopt" -d "Set the graphics mode"

# brightness completion
complete -c $progname -n "$keyboard_options" -d "available brightness modes" -a "$brightness_modes"

# fan completion
complete -c $progname -n "$fan_options" -d "available fan modes" -a $fan_modes

# graphics completion
set -l gopt 'not __fish_contains_opt -s h -s g -s m -s p help mode get pow;'

complete -c $progname -n "$graphics_options and $gopt" -a "-h" -d "print help message"
complete -c $progname -n "$graphics_options and $gopt" -a "-g" -d "Get the current mode"

complete -c $progname -s h -f -l help -n "$graphics_options and $gopt" -d "print help message"
complete -c $progname -s m -f -l mode -n "$graphics_options and $gopt" -d "Set graphics mode: <nvidia, hybrid, compute, integrated>"
complete -c $progname -s g -f -l get -n "$graphics_options and $gopt" -d "Get the current mode"
complete -c $progname -s p -f -l pow -n "$graphics_options and $gopt" -d "Get the current power status"

complete -c $progname -n "$graphics_options and $gmod_options" -d "available graphics modes" -a "$graphics_modes"

# led-mode completion
complete -c $progname -n "$led_options" -a "-h" -d "print help message"
complete -c $progname -n "$led_options" -a "-n" -d "switch to next aura mode"

complete -c $progname -s h -f -l help -n "$led_options" -d "print help message"
complete -c $progname -s n -f -l next-mode -n "$led_options" -d "switch to nex aura mode"
complete -c $progname -s p -f -l prev-mode -n "$led_options" -d "switch to previous aura mode"
complete -c $progname -n "$led_options" -d "available led modes" -a "$led_modes"

# profile completion
set -l popt 'not __fish_contains_opt -s h -s n -s c -s t -s m -s M -s f help next create turbo min-percentage max-percentage fan-preset;'

complete -c $progname -n "$profile_options and $popt" -a "-h" -d "print help message"
complete -c $progname -n "$profile_options and $popt" -a "-n" -d "toggle to next profile in list"

complete -c $progname -s h -f -l help -n "$profile_options and $popt" -d "print help message"
complete -c $progname -s n -f -l next -n "$profile_options and $popt" -d "toggle to next profile in list"
complete -c $progname -s c -f -l create -n "$profile_options and $popt" -d "create the profile if it doesn't exist"
complete -c $progname -s t -f -l turbo -n "$profile_options and $popt" -d "enable or disable cpu turbo"
complete -c $progname -s m -f -l min-percentage -n "$profile_options and $popt" -d "set min cpu scaling (intel)"
complete -c $progname -s M -f -l max-percentage -n "$profile_options and $popt" -d "set max cpu scaling (intel)"
complete -c $progname -s f -f -l fan-preset -n "$profile_options and $popt" -d "<silent, normal, boost>"
complete -c $progname -n "$profile_option and __fish_contains_opt fan-preset" -d "available fan modes" -a $fan_modes

