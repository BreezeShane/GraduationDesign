import { Button, Card, Input, Modal, UploadFile } from 'antd';
import { BookOutlined } from '@ant-design/icons';
import { useState } from 'react';
import React from 'react';
import { NotificationInstance } from 'antd/es/notification/interface';
import UploadImage from '../../Componets/UploadImage';
import axios from 'axios';
import ResultPagePanel from '../../Componets/ResultPagePanel';

const Common: React.FC<{ messageClient: NotificationInstance }> = (props) => {
    const { messageClient } = props;
    const [open, setOpen] = useState(false);
    const [labelList, setLabelList] = useState("");
    const [fileList, setFileList] = useState<UploadFile[]>([]);
    const [result_table, setResultTable] = useState({});

    const handleOpenModal = () => {
        setOpen(true);
    }

    const handleOk = () => {
        if (!sessionStorage.getItem('useremail') || fileList.length == 0) {
            messageClient.error({
                message: `Failed to submit feedback!`,
                description: "You should sign in first and upload at least one image file!",
                placement: 'topLeft',
                duration: 2,
            });
            return;
        }
        let label_list = labelList.split(';');
        for (var index in fileList) {
            let file = fileList[index]
            let real_label = null;
            if (label_list.length > 0){
                try{
                    if (label_list[index]){
                        real_label = label_list[index]
                    }
                } catch {}
            }
            let post_body = {
                useremail: sessionStorage.getItem('useremail'),
                real_label: real_label,
                pic_name: file.name
            };
            axios.post('/user/subm_fb', post_body)
            .then(function (res) {
                messageClient.success({
                    message: `Succeeded to submit feedback!`,
                    description: `Thank you very much for your precious feedback! Returned response: ${res}`,
                    placement: 'topLeft',
                    duration: 2,
                });
            }).catch((err) => {
                messageClient.error({
                    message: `Failed to submit feedback!`,
                    description: `The feedback weren't sent(Error Responce: {${err}}), please try it again later! Thank you for your patience!`,
                    placement: 'topLeft',
                    duration: 2,
                });
            });
        }
    }

    const handleCancel = () => {
    setOpen(false);
    };

    const handleChange = (value: any) => {
        setLabelList(value.target.value);
    };

    const handleClearFiles = () => {
        setFileList([]);
    };

    const handlePredictImages = () => {
        let file_list = [];
        for (var idx in fileList) {
            let file = fileList[idx];
            file_list.push(file.name);
        }
        axios.post("/user/infer", {
            useremail: sessionStorage.getItem('useremail'),
            file_list: JSON.stringify(file_list),
        }).then(function (res) {
            messageClient.success({
                message: `Succeeded to Infer images!`,
                description: `Thank you for your use!`,
                placement: 'topLeft',
                duration: 2,
            });
            setResultTable(res.data);
        }).catch((err) => {
            messageClient.error({
                message: `Failed to infer images!`,
                description: `The images might got wrong, please try it again later! Thank you for your patience! Returned response: ${err}`,
                placement: 'topLeft',
                duration: 2,
            });
        });
    };

    return (
        <div style={{ display: "flex", width: "100%", textAlign: "center" }}>
            <div style={{width: '50%'}}>
                <Card title='图片上传'>
                    <UploadImage messageClient={messageClient} fileList={fileList} setFileList={setFileList} />
                </Card>
            </div>

            <div style={{width: '50%'}}>
                <div style={{ width: "100%", height: "10%", textAlign: "center", display: "flex" }}>
                    <div style={{ width: "50%" }}>
                        <Button onClick={handlePredictImages}>Predict Images</Button>
                    </div>
                    <div style={{ width: "50%" }}>
                        <Button onClick={handleClearFiles}>Clear Files</Button>
                    </div>
                </div>
                <ResultPagePanel result_table={result_table} />
                <div style={{ height: "10%", display: "flex", width: "100%", justifyContent: "center" }}>
                    <p style={{ display: "flex",  textAlign: 'center', alignItems: "center", height: "100%" }}>
                        Incorrect Result? Please give
                    </p>
                    <div style={{ width: "20%", textAlign: "center" }}>
                        <Button style={{ height: "100%" }} onClick={handleOpenModal} >Feedback</Button>
                        <Modal title="Give Feedback"
                            open={open}
                            onOk={handleOk}
                            onCancel={handleCancel}
                        >
                            <Input
                                size="large"
                                prefix={<BookOutlined />}
                                placeholder="Use ';' to spilt labels of images from up to down."
                                onChange={handleChange}
                                allowClear={true}
                            />
                        </Modal>
                    </div>
                    <p style={{ display: "flex",  textAlign: 'center', alignItems: "center", height: "100%" }}>
                        !
                    </p>
                </div>
            </div>
        </div>
    );
}

export default Common;