"""
    Compile the trained model using TVM.
"""
import os
import tvm
import torch
import timeit
import numpy as np
from tvm import relay, autotvm
from tvm.contrib import utils, graph_executor
from torchvision import transforms
import tvm.auto_scheduler as auto_scheduler
from tvm.autotvm.tuner import XGBTuner, GATuner, RandomTuner, GridSearchTuner

from ModelTransfer.Classifier import Classifier
from TransferProcedures.compile_utils import load_classifier_model, transfer_classifier_model
from config import MODEL_TUNING_JSON, COMPILED_MODEL_DIR

def compile_model(args):
    """ Compile the trained model by TVMC while using tune, etc. """
    if None in (args.target, args.mod2cmp, args.save_path):
        raise ValueError("All params '--model_path, --target, --save_path' are required!")

    if not os.path.isfile(args.mod2cmp):
        raise ValueError(
            f"The path to model is required, but got {args.mod2cmp} which is not file!")
    print("Loading model...")
    if args.ts:
        pass
    else:
        model = load_classifier_model(pretrained_model_path=args.mod2cmp, device=args.device)
        scripted_model = torch.jit.trace(model, torch.rand(1, 3, 512, 512)).eval()

        input_name = "input_img"
        shape_list = [(input_name, (1, 3, 512, 512))]

        mod, params = relay.frontend.from_pytorch(scripted_model, shape_list)
        target = tvm.target.Target(args.target)

        tune_model(mod=mod, target=target, params=params)

        module_libs = []
        with tvm.transform.PassContext(opt_level=3):
            lib = relay.build(mod, target=target, params=params)
            module_libs.append( ("unoptimized", lib) )

        with autotvm.apply_history_best(MODEL_TUNING_JSON):
            with tvm.transform.PassContext(opt_level=3, config={}):
                lib = relay.build(mod, target=target, params=params)
                module_libs.append( ("optimized", lib) )

        for (opt_status, lib) in module_libs:
            save_module(lib=lib, lib_name=opt_status)

def test_module(args):
    """ Definition of testing modules and Compare optimized and unoptimized. """
    if args.target is None:
        raise ValueError("The params '--target' is required!")

    timing_repeat = 10
    timing_number = 3
    dev = tvm.device(str(args.target), 0)
    output_shape = (1, 1, 102)

    modules_prefix = ["unoptimized", "optimized"]
    for prefix_name in modules_prefix:
        # loaded_json = open(temp.relpath(f"{prefix_name}_deploy_graph.json"), encoding="utf-8").read()
        path_lib = os.path.join(COMPILED_MODEL_DIR, f"{prefix_name}_deploy_lib.tar")

        loaded_lib = tvm.runtime.load_module(path_lib)
        # loaded_params = bytearray(open(temp.relpath(f"{prefix_name}_deploy_param.params"), "rb").read())
        input_data = tvm.nd.array(np.random.uniform(size=(1, 3, 512, 512)).astype("float32"))

        module = graph_executor.GraphModule(loaded_lib["default"](dev))

        module.set_input("input_img", input_data)
        module.run()
        tvm_output = module.get_output(0, tvm.nd.empty(output_shape)).numpy()

        label = np.argmax(tvm_output)
        print (f"Succeeded to test the {prefix_name} module, the run result is: {label}")

        time_cost = (
            np.array(timeit.Timer(lambda: module.run())
                .repeat(repeat=timing_repeat, number=timing_number))
            * 1000 / timing_number
        )
        scores_dict = {
            "mean": np.mean(time_cost),
            "median": np.median(time_cost),
            "std": np.std(time_cost),
        }
        print(f"The {prefix_name} module spent time: {scores_dict}")

def save_module(lib, lib_name):
    """ Definition of saving module lib for loading. """
    save_dir = COMPILED_MODEL_DIR
    path_lib = os.path.join(save_dir, f"{lib_name}_deploy_lib.tar")
    lib.export_library(path_lib)
    # with open(os.path.join(save_dir, f"{lib_name}_deploy_graph.json"), "w", encoding='utf-8') as fo:
    #     fo.write(graph)
    # with open(os.path.join(save_dir, f"{lib_name}_deploy_param.params"), "wb") as fo:
    #     fo.write(relay.save_param_dict(params))

def tune_model(mod, target, params, tuner="xgb"):
    """ Definition of tuning model """
    print("Tuning the modules...")
    number = 10
    repeat = 1
    min_repeat_ms = 0  # since we're tuning on a CPU, can be set to 0
    timeout = 10  # in seconds

    runner = autotvm.LocalRunner(
        number=number,
        repeat=repeat,
        timeout=timeout,
        min_repeat_ms=min_repeat_ms,
        enable_cpu_cache_flush=True,
    )

    tuning_option = {
        "tuner": "xgb",
        "trials": 10, # For CPU we(Apache) recommend 1500, for GPU 3000-4000.
        "early_stopping": 100,
        "measure_option": autotvm.measure_option(
            builder=autotvm.LocalBuilder(build_func="default"), runner=runner
        ),
        "tuning_records": MODEL_TUNING_JSON,
    }

    tasks = autotvm.task.extract_from_program(mod["main"], target=target, params=params)

    for i, task in enumerate(tasks):
        prefix = "[Task %2d/%2d] " % (i + 1, len(tasks))
        tuner_obj = get_tuner(tuner, task)

        tuner_obj.tune(
            n_trial=min(tuning_option["trials"], len(task.config_space)),
            early_stopping=tuning_option["early_stopping"],
            measure_option=tuning_option["measure_option"],
            callbacks=[
                autotvm.callback.progress_bar(tuning_option["trials"], prefix=prefix),
                autotvm.callback.log_to_file(tuning_option["tuning_records"]),
            ],
        )

def get_tuner(tuner, task):
    """ Create Tuner """
    if tuner == "xgb":
        tuner_obj = XGBTuner(task, loss_type="reg")
    elif tuner == "xgb_knob":
        tuner_obj = XGBTuner(task, loss_type="reg", feature_type="knob")
    elif tuner == "xgb_itervar":
        tuner_obj = XGBTuner(task, loss_type="reg", feature_type="itervar")
    elif tuner == "xgb_curve":
        tuner_obj = XGBTuner(task, loss_type="reg", feature_type="curve")
    elif tuner == "xgb_rank":
        tuner_obj = XGBTuner(task, loss_type="rank")
    elif tuner == "xgb_rank_knob":
        tuner_obj = XGBTuner(task, loss_type="rank", feature_type="knob")
    elif tuner == "xgb_rank_itervar":
        tuner_obj = XGBTuner(task, loss_type="rank", feature_type="itervar")
    elif tuner == "xgb_rank_curve":
        tuner_obj = XGBTuner(task, loss_type="rank", feature_type="curve")
    elif tuner == "xgb_rank_binary":
        tuner_obj = XGBTuner(task, loss_type="rank-binary")
    elif tuner == "xgb_rank_binary_knob":
        tuner_obj = XGBTuner(task, loss_type="rank-binary", feature_type="knob")
    elif tuner == "xgb_rank_binary_itervar":
        tuner_obj = XGBTuner(task, loss_type="rank-binary", feature_type="itervar")
    elif tuner == "xgb_rank_binary_curve":
        tuner_obj = XGBTuner(task, loss_type="rank-binary", feature_type="curve")
    elif tuner == "ga":
        tuner_obj = GATuner(task, pop_size=50)
    elif tuner == "random":
        tuner_obj = RandomTuner(task)
    elif tuner == "gridsearch":
        tuner_obj = GridSearchTuner(task)
    else:
        raise ValueError("Invalid tuner: " + tuner)
    return tuner_obj
        # dev = tvm.device(str(target), 0)
    # model.summary()
    # # model.save(desired_model_path) # not needed temporarily.
    # if args.tune:
    #     tvmc.tune(model, target=args.target,
    #         enable_autoscheduler=args.autoscheduler,
    #         trials=args.trails,
    #         timeout=args.timeout,
    #         prior_records=TUNING_RECORDS_PATH
    #             if os.path.exists(TUNING_RECORDS_PATH) and args.continue_compile else None
    #     )

    #     _package = tvmc.compile(model, target=args.target,
    #         package_path=args.pkg_path, tuning_records = TUNING_RECORDS_PATH)
    # else:
    #     _package = tvmc.compile(model, target=args.target, package_path=args.pkg_path)
