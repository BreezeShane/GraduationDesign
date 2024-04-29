import { Button, Card, Input, Modal } from 'antd';
import { BookOutlined } from '@ant-design/icons';
import { ChangeEventHandler, ReactNode, useState } from 'react';
import React from 'react';
import { NotificationInstance } from 'antd/es/notification/interface';
import UploadImage from '../Componets/UploadImage';
import axios from 'axios';

const Common: React.FC<{ messageClient: NotificationInstance }> = (props) => {
    const { messageClient } = props;
    const [open, setOpen] = useState(false);
    const [realLabel, setLabel] = useState("");
    const [fileName, setFileName] = useState(new File([], ""));

    const handleOpenModal = () => {
        setOpen(true);
    }

    const handleOk = () => {
        if (!sessionStorage.getItem('useremail') || !fileName.name) {
            messageClient.error({
                message: `Forbidden Operation!`,
                description: "You should sign in first!",
                placement: 'topLeft',
                duration: 2,
            });
        }
        let post_body = {
            useremail: sessionStorage.getItem('useremail'),
            real_label: realLabel,
            pic_name: fileName.name
        }
        axios.post('/')
        console.log(post_body)
    }

    const handleCancel = () => {
    setOpen(false);
    };

    const handleChange = (value: any) => {
        setLabel(value.target.value);
    };
    
    return (
        <div style={{ display: "flex" }}>
            <div style={{width: '50%'}}>
                <Card title='图片上传'>
                    <UploadImage setFileName={setFileName} />
                </Card>
            </div>
            <div style={{width: '50%'}}>
                <div style={{ height: "10%" }}>
                    <a id='SourceLink' href=''></a>
                </div>
                <div style={{ height: "80%" }}>
                    <h1 id='SearchTitle'>Title</h1>
                    <p id='SearchContent'>Content</p>
                </div>
                <div style={{ height: "10%" }}>
                    Incorrect Result? Please give 
                    <div>
                        <Button onClick={handleOpenModal} >Feedback</Button>
                        <Modal title="Give Feedback"
                            open={open}
                            onOk={handleOk}
                            onCancel={handleCancel}
                        >
                            <Input
                                size="large"
                                prefix={<BookOutlined />}
                                onChange={handleChange}
                                allowClear={true}
                            />
                        </Modal>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default Common;