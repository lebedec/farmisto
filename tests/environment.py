from testing import Context, setup_environment, setup_scenario, teardown_scenario


def before_all(context: Context):
    setup_environment(context)


def before_scenario(context: Context, s):
    print(type(s))
    setup_scenario(context)


def after_scenario(context: Context, s):
    print(type(s))
    teardown_scenario(context)
