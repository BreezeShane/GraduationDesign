import { Input, Popover } from "antd";
import { NotificationInstance } from "antd/es/notification/interface";
import commands from "./commands.json";
import copy from "copy-to-clipboard";
import { BaseSyntheticEvent } from "react";
import { InfoCircleOutlined } from "@ant-design/icons";
import { kMaxLength } from "buffer";

const { TextArea } = Input;

const content = (
    <p>
        <InfoCircleOutlined /> Double click to copy
    </p>
);

const NormalDivider = (
    <>
        <br/><br/>
    </>
);

const SmallDivider = (
    <>
        <br/>
    </>
);

const TextAreaProperties = {
    rows: 3,
}

const Commands: React.FC<{ messageClient: NotificationInstance }> = (props) => {
    const { messageClient } = props;

    const handleDoubleClickCopy = (event: BaseSyntheticEvent) => {
        let text_clicked = event.target.value;
        if (copy(text_clicked)){
            messageClient.success({
                message: `Command Copied!`,
                description: "You could paste the command to WebSSH!",
                placement: 'topLeft',
                duration: 2,
            });
        } else {
            messageClient.error({
                message: `Failed to copy command!`,
                description: "You could copy the command manually!",
                placement: 'topLeft',
                duration: 2,
            });
        }
    };

    return (
        <div style={{ display: "flex" }}>
            <div style={{ width: "50%" }}>
                <Popover content={content} trigger="hover">
                    Train Command: <TextArea style={{ width: "90%"}} onDoubleClick={handleDoubleClickCopy} value={commands.train} {...TextAreaProperties} />
                </Popover>
                {NormalDivider}
                <Popover content={content} trigger="hover">
                    Train with extra tools Command: <TextArea style={{ width: "90%"}} onDoubleClick={handleDoubleClickCopy} value={commands.new_train} {...TextAreaProperties} />
                </Popover>
                {NormalDivider}
                <Popover content={content} trigger="hover">
                    Infer Command: <TextArea style={{ width: "90%"}} onDoubleClick={handleDoubleClickCopy} value={commands.infer} {...TextAreaProperties} />
                </Popover>
            </div>
            <div style={{ width: "50%" }}>
                <Popover content={content} trigger="hover">
                    Validate Command: <TextArea style={{ width: "90%"}} onDoubleClick={handleDoubleClickCopy} value={commands.valid} {...TextAreaProperties} />
                </Popover>
                {NormalDivider}
                <Popover content={content} trigger="hover">
                    Compile Model Command: <TextArea style={{ width: "90%"}} onDoubleClick={handleDoubleClickCopy} value={commands.compile} {...TextAreaProperties} />
                </Popover>
                {NormalDivider}
                <Popover content={content} trigger="hover">
                    Help Command: <Input style={{ width: "90%"}} onDoubleClick={handleDoubleClickCopy} value={commands.help} />
                </Popover>
                {SmallDivider}
                <Popover content={content} trigger="hover">
                    List Targets Compiling Model Supports Command: <Input style={{ width: "90%"}} onDoubleClick={handleDoubleClickCopy} value={commands.list_target} />
                </Popover>
                {SmallDivider}
                <Popover content={content} title="Title" trigger="hover">
                    Show Train Graphs Command: <Input style={{ width: "90%"}} onDoubleClick={handleDoubleClickCopy} value={commands.show_graphs} />
                </Popover>
            </div>
        </div>
    );
}

export default Commands;