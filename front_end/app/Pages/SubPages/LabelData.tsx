import React, { BaseSyntheticEvent, ReactNode, useEffect, useRef, useState } from 'react';
import { Button, Carousel, Input, Space, Image, PaginationProps, Pagination, Spin } from 'antd';
import { BookOutlined } from '@ant-design/icons';
import { NotificationInstance } from 'antd/es/notification/interface';
import axios from 'axios';

const image_src_prefix = 'data:image/jpeg;base64,';
const fallback_image = "data:image/png;base64,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";

const contentStyle: React.CSSProperties = {
  margin: 0,
  height: '160px',
  color: '#fff',
  lineHeight: '160px',
  textAlign: 'center',
  background: '#364d79',
};

const arrayBufferToBase64 = (image_buffer: ArrayBuffer) => {
    var binary = '';
    var bytes = new Uint8Array(image_buffer);
    var len = bytes.byteLength;
    for (var i = 0; i < len; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return window.btoa(binary);
}

const generate_image_src = (image_buffer: ArrayBuffer) => {
    return image_src_prefix + arrayBufferToBase64(image_buffer);
}

const LabelData: React.FC<{ messageClient: NotificationInstance }> = (props) => {
    const { messageClient } = props;
    const default_page = 0;
    const [imageLabel, setImageLabel] = useState("");
    const [image_list, setImageList] = useState<string[]>([]);
    const [total, setTotal] = useState(0);
    const [current_page, setCurrentPage] = useState(0);
    const [current_image_link, setCurrentImageLink] = useState("");

    useEffect(() => {
        axios.get("/user/label_pic", {
            params: {
                email: sessionStorage.getItem("useremail"),
            }
        }).then((res) => {
            let images: string[] = [];
            for (let item in res.data) {
                images.push(res.data[item].pic_link);
            }
            setImageList([...images])
            setTotal(res.data.length);
            axios.get("/fetch_image", {
                params: {
                    useremail: sessionStorage.getItem('useremail'),
                    image_name: images[default_page]
                },
                responseType: 'arraybuffer',
            }).then((res) => {
                setCurrentImageLink(generate_image_src(res.data))
            })
        }).catch((err) => {
            console.log(err);
        })
    }, []);

    const handleChange = (value: BaseSyntheticEvent) => {
        setImageLabel(value.target.value);
    };

    const updateImagebyIndex = (page_num: number) => {
        let idx = page_num - 1;
        axios.get("/fetch_image", {
            params: {
                useremail: sessionStorage.getItem('useremail'),
                image_name: image_list[idx]
            },
            responseType: 'arraybuffer',
        }).then((res) => {
            setCurrentImageLink(generate_image_src(res.data))
        });
        // if (table.length > 0){
        //     setCurrentImageLink(table[idx]);
        // }
    };

    const onChange: PaginationProps['onChange'] = (page) => {
        setCurrentImageLink("");
        setCurrentPage(page);
        updateImagebyIndex(page);
    };

    const handleSubmit = () => {
        let idx = current_page - 1;
        axios.post("/user/label_pic", {
            useremail: sessionStorage.getItem('useremail'),
            image_name: image_list[idx],
            image_label: imageLabel,
        }).then((res) => {
            console.log(res)
        });
    }

    const handleClear = () => {
        setImageLabel("");
    }

    return (
        <>
            <div style={{ textAlignLast: "center" }}>
                {
                    !current_image_link &&
                    <div style={{ display: "flex", height: "80vh", alignItems: "center", justifyContent: "center" }}>
                        <Spin size="large" tip="Loading...">
                            Fecthing the image...
                        </Spin>
                    </div>
                }
                {
                    current_image_link &&
                    <Image
                        height={"80vh"}
                        placeholder={true}
                        src={current_image_link}
                        fallback={fallback_image}
                    />
                }
                <br/>
                <Pagination
                    style={{ height: "6vh", display: "flex", alignItems: "center", justifyContent: "center" }}
                    total={total}
                    showQuickJumper
                    showTotal={(total, range) => `${range[0]} of ${total} items`}
                    defaultCurrent={default_page}
                    defaultPageSize={1}
                    current={current_page}
                    onChange={onChange}
                />
                <Space style={{ height: "6vh" }}>
                    <Input
                        size="large"
                        prefix={<BookOutlined />}
                        placeholder="Input the label you think correct"
                        value={imageLabel}
                        onChange={handleChange}
                        allowClear={true}
                    />
                    <Button onClick={handleSubmit}>Submit</Button>
                    <Button onClick={handleClear} danger>Clear</Button>
                </Space>
            </div>
        </>
    );
};

export default LabelData;