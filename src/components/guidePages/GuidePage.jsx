import { useEffect, useState } from "react"
import { useForm } from "react-hook-form"
import snapshort1 from "../../assets/snapshot1.png"
import snapshot2 from "../../assets/snapshort2.png"
import snapshot3 from "../../assets/snapshort3.png"
import snapshort4 from "../../assets/snapshort4.png"


function GuidePage(){
    const [page, setPage] = useState(0)



    const handleClick = () => {
        setPage(p => p + 1)
        console.log(page)
    }

    const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm()

  const onSubmit = (data) => {
    console.log(data)
    localStorage.setItem("name", data.name)
    console.log(localStorage.getItem("name"))
    handleClick()
}


    if(page === 0){
    return(
        <div style={{
                backgroundImage: "radial-gradient(circle, rgba(39, 39, 42, 1) 1.5px, transparent 1px)",
                backgroundSize: "20px 20px",
                backgroundRepeat: "repeat",
                }}
            className="flex flex-col justify-center items-center gap-1 h-screen bg-[#3D3C3C] font-sans">
            <div className="flex flex-row justify-center text-6xl pb-5 font-bold">Pathfinder</div>
            <div className="felx flex-row justify-center text-center text-3xl">A tool to make your life easy</div>
            <button onClick={handleClick} type="button" className="absolute bottom-3 right-5 flex flex-row justify-center py-1 px-3 bg-black rounded-md text-center text-white ">next</button>
        </div>
    )
}

if(page === 1){
    return(
        <div className="flex flex-col justify-center items-center gap-5 h-screen bg-[#3D3C3C] font-sans">
            <div className="text-4xl">Enter Your Name</div>
            <div>
                <form onSubmit={handleSubmit(onSubmit)}>
                    <input
                    className="bg-white rounded-md text-2xl text-center"
                    defaultValue="" {...register("name")} type='text' placeholder="Type"/>
                </form>
            </div>
                <button type='button' onClick={handleClick} className="absolute bottom-3 right-5 flex flex-row justify-center py-1 px-3 bg-black rounded-md text-center text-white ">next</button>
        </div>
    )
}

if(page === 2){
    return(
        <div style={{
                backgroundImage: "radial-gradient(circle, rgba(39, 39, 42, 1) 1.5px, transparent 1px)",
                backgroundSize: "20px 20px",
                backgroundRepeat: "repeat",
                }}
            className="flex flex-col justify-center gap-5 p-5 h-screen bg-[#3D3C3C] font-sans text-white">
            <div className="felx flex-row justify-left text-left text-4xl ">Hi ,{localStorage.getItem("name")} Welcome to Pathfinder</div>
            <div className="text-left backdrop-blur-sm">
                A powerful Raycast-inspired launcher application built with Tauri and React. PathFinder provides instant access to your most-used tools and information through a beautiful, keyboard-driven interface. And you can access it just my pressing 
                </div>
            <div className="text-5xl text-center text-[#737373]">Ctrl+Shift+Space</div>
            <button type="button" onClick={handleClick} className="absolute bottom-3 left-180 flex flex-row justify-center py-1 px-3 bg-black rounded-md text-center text-white ">next</button>
        </div>
    )
}

if(page === 3){

    return(
        <div className="flex flex-col justify-center gap-5 p-5 h-full bg-[#3D3C3C] font-sans text-white">
            <div className="flex flex-row justify-left text-left text-4xl ">Clipboard:</div>
            <div className="text-left backdrop-blur-sm">
                A clipboard manager feature that displays all copied items with details like timestamp, size, and usage count. Users can easily view, search, and re-copy any saved entry, making it simple to manage frequently used texts or data efficiently.
            </div>
            <div className="m-5 px-10 pt-10 rounded-t-xl bg-[#929292]">
                <img src={snapshort1} alt='snapshot1'
                className="rounded-t-xl "/>
            </div>

            <div className="text-left mt-5 backdrop-blur-sm">
                A clipboard search feature that lets users quickly find any saved clipboard entry by typing keywords. It filters items in real-time, showing relevant results with details like timestamp, size, and usage, enabling fast retrieval and efficient clipboard management.                </div>
            <div className="m-5 px-10 pt-10 rounded-t-xl bg-[#929292]">
                <img src={snapshot2} alt='snapshot2'
                className="rounded-t-xl "/>
            </div>

            <div className="flex flex-row justify-end">
            <button onClick={handleClick} className=" py-1 px-3 bg-black rounded-md text-center text-white ">next</button>
            </div>
        </div>
    )
}

if(page === 4){

    return(
            <div className="flex flex-col justify-center gap-5 p-5 h-full bg-[#3D3C3C] font-sans text-white">
            <div className="flex flex-row justify-left text-left text-4xl ">Online Search:</div>
            <div className="text-left backdrop-blur-sm">
                Quick-access web search allows users to instantly search the internet directly from the app. With a streamlined interface, it supports rapid queries, displays results efficiently, and saves frequently used searches, enabling fast, convenient, and productive online information retrieval.                </div>
            <div className="m-5 px-10 pt-10 rounded-t-xl bg-[#929292]">
                <img src={snapshot3} alt='snapshot1'
                className="flex justify-center items-center rounded-t-xl "/>
            </div>
            <div className="flex flex-row justify-end">
            <button onClick={handleClick} className=" py-1 px-3 bg-black rounded-md text-center text-white ">next</button>
            </div>
        </div>
    )
}

if(page === 5){

    return(

        <div className="flex flex-col justify-center gap-5 p-5 h-full bg-[#3D3C3C] font-sans text-white">
            <div className="flex flex-row justify-left text-left text-4xl ">Open File:</div>
            <div className="text-left backdrop-blur-sm">
                A fast file access feature that lets users locate and open files instantly. By typing filenames or keywords, it quickly filters results, showing file paths and details, streamlining workflow and saving time when managing documents, media, or system files.                </div>
            <div className="m-5 px-10 pt-10 rounded-t-xl bg-[#929292]">
                <img src={snapshort4} alt='snapshot1' className="rounded-t-xl "/>
            </div>
            <div className="flex flex-row justify-end">
            <button onClick={handleClick} className=" py-1 px-3 bg-black rounded-md text-center text-white ">next</button>
            </div>        </div>
    )
}

if(page === 6){

    return(
        <div className="flex flex-row gap-3 justify-center items-center h-screen w-screen bg-[#3D3C3C]">
        <div className="flex flex-row text-center text-3xl">
            All done, press <div className="flex justify-center items-center px-2 mx-2 border border-b-2 border-r-2 rounded-md bg-zinc-800">esc</div>to continue to 
        </div>
        <div className="py-2 px-3 bg-black rounded-md text-center text-white text-3xl">Pathfinder</div>
        </div>
    )
}







}
export default GuidePage