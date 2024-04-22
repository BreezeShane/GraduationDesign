import os
from tvm.driver import tvmc

TUNING_RECORDS_PATH = ".records.log"

def compile_model(args):
    if args.cmp_mode is not None:
        raise ValueError("The parameter '--compile_mode' is needed!")
    if not os.path.isfile(args.mod2cmp):
        raise ValueError(f"The path to model is needed, but got {args.mod2cmp} which is not file!")
    model = tvmc.load(args.mod2cmp, shape_dict={
        'input1' : [1, 2, 3, 4], 
        'input2' : [1, 2, 3, 4]
    }) # the shape_dict comes from 'input/shape_dict' loaded by netron
    model.summary()
    # model.save(desired_model_path) # not needed temporarily.
    tuning_config = {
        'trials': 10000,
        'timeout': 10
    }
    if args.tune:
        tvmc.tune(model, target=args.cmp_mode, 
            enable_autoscheduler=args.autoscheduler, 
            trials=args.trails,
            timeout=args.timeout,
            prior_records=TUNING_RECORDS_PATH 
                if os.path.exists(TUNING_RECORDS_PATH) and args.continue_compile else None
        )

        package = tvmc.compile(model, target=args.cmp_mode, 
            package_path=args.pkg_path, tuning_records = TUNING_RECORDS_PATH)
    else:
        package = tvmc.compile(model, target=args.cmp_mode, package_path=args.pkg_path)