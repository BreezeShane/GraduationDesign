import torch
from os.path import exists, isfile, join

from dl_svc.datasetloader import load_dataset
from dl_svc.Loss.contrastive_loss_with_temperature import ContrastiveLossWithTemperature

def validate(args):
    if args.vset is None:
        raise ValueError("Validate Dataset is needed!")
    v_dataloader = load_dataset(args.vset)

    model = __load_model(args=args)
    loss_criterion = ContrastiveLossWithTemperature(
        logit_scale = math.log(1 / 0.07), # DEFAULT_LOGIT_SCALE
        logit_scale_min = math.log(1.0),
        logit_scale_max = math.log(100.0),
    )

    with torch.no_grad():
        valid_epoch_loss = []
        acc, nums = 0., 0
        
        for idx,(label, inputs) in enumerate(tqdm(valid_dataloader)):
            inputs = inputs.to(torch.float32).to(args.device)
            label = label.to(torch.float32).to(args.device)
            outputs = model(inputs)
            loss = loss_criterion(outputs, label)

            valid_epoch_loss.append(loss.item())
            acc += sum(outputs.max(axis=1)[1] == label).cpu()
            nums += label.size()[0]

        E_v_loss = np.average(valid_epoch_loss)
        Acc_v = 100 * acc / nums
        valid_epochs_loss.append(E_v_loss)
        val_acc.append(Acc_v)
        print(
            "Valid Acc = {:.3f}%, loss = {}"
            .format(Acc_v, E_v_loss)
        )

def inference(args):
    if args.iset is None:
        raise ValueError("Inference Dataset is needed!")
    i_dataloader = load_dataset(args.iset, batch_size=1)
    model = __load_model(args=args)
    loss_criterion = ContrastiveLossWithTemperature(
        logit_scale = math.log(1 / 0.07), # DEFAULT_LOGIT_SCALE
        logit_scale_min = math.log(1.0),
        logit_scale_max = math.log(100.0),
    )

    result = []
    with torch.no_grad():
        for idx, (_, img_input) in enumerate(tqdm(i_dataloader)):
            img_input = img_input.to(torch.float32).to(args.device)
            output = model(img_input)

            label=torch.argmax(output, 1)
            result.append(label)
    
    print(result)


def __load_model(args):
    model_path = args.mod_loc
    if not exists(model_path):
        raise IOError(f"Trained model not found! Attempting to load: {model_path}")
    if not isfile(model_path):
        raise ValueError("Trained model should be a file!")
    
    if args.gpu_id is not None:
        model_dict = torch.load(model_path, map_location=torch.device(f'cuda:{args.gpu_id}'))
    else:
        model_dict = torch.load(model_path, map_location=torch.device('cpu'))
    try:
        match model_dict['name']:
            case 'coca_vit_l_14':
                model = coca_vit_l_14()
            case 'coca_vit_b_32':
                model = coca_vit_b_32()
            case 'custom_coca_vit':
                pass
    except KeyError:
        raise KeyError(f"Not compatible model! Get model here: {model_path}")

    model.load_state_dict(model_dict['model'])
    model.eval()

    return model